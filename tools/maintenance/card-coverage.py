"""Which lines of a card's Rust file its scenarios never execute.

    python tools/maintenance/card-coverage.py "Card Name"
    tools\\card-coverage.cmd "Card Name"

Runs the card's scenarios (`corpus --card`, every scenario whose decks hold the card) under cargo-llvm-cov, then reads
the line coverage of crates/cantrip-cards/src/cards/<card>.rs. Prints each line that never ran, with its text, and
exits 1 if there is one, 0 when every line ran. A card with no Rust file has nothing to cover. Needs cargo-llvm-cov
(`rustup component add llvm-tools-preview`, `cargo install cargo-llvm-cov --locked`). The add-cards skill requires
every line covered.
"""

import os
import re
import subprocess
import sys
import tempfile
import unicodedata
import uuid
from pathlib import Path

from card_tool_runtime import build_env, build_lock, workspace_root

ROOT = workspace_root(__file__)
CARDS = os.path.join(ROOT, "crates", "cantrip-cards", "src", "cards")


def file_stem(name):
    """The card's Rust file name: new-card.py's rule (accents folded, other non-alphanumerics to '_')."""
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.replace("'", "").lower()).strip("_")


def card_file(name):
    for stem in (file_stem(name), re.sub(r"[^a-z0-9]+", "_", name.replace("'", "").lower()).strip("_")):
        path = os.path.join(CARDS, stem + ".rs")
        if os.path.isfile(path):
            return path
    return None


def const_fn_lines(source):
    """The line numbers inside `const fn` items: they run at compile time, so llvm-cov counts them as never run."""
    lines, i = set(), 0
    while i < len(source):
        if re.match(r"\s*(pub(\([^)]*\))?\s+)?const\s+fn\s", source[i]):
            depth, started, j = 0, False, i
            while j < len(source):
                code = re.sub(r'"(\\.|[^"\\])*"', "", source[j]).split("//")[0]
                depth += code.count("{") - code.count("}")
                started = started or "{" in code
                lines.add(j + 1)
                if started and depth <= 0:
                    break
                j += 1
            i = j
        i += 1
    return lines


def coverage_lcov(name):
    """Fresh counters over compatible instrumented artifacts, serialized by the caller."""
    env = build_env(ROOT)
    coverage_dir = Path(env["CARGO_TARGET_DIR"]) / "llvm-cov-target"
    env["CARGO_LLVM_COV_TARGET_DIR"] = str(coverage_dir)
    identity = "card-coverage-" + uuid.uuid4().hex
    env["LLVM_PROFILE_FILE_NAME"] = identity + "-%p-%m.profraw"

    def command(args, **kwargs):
        proc = subprocess.run(args, cwd=ROOT, env=env, capture_output="stdout" not in kwargs,
                              text=True, encoding="utf-8", errors="replace", **kwargs)
        if proc.returncode:
            raise RuntimeError((proc.stdout or "")[-4000:] + (proc.stderr or "")[-4000:])
        return proc

    # --no-report already implies --no-clean in cargo-llvm-cov; specifying both
    # is rejected. Explicitly clear only counters, retaining compatible builds.
    command(["cargo", "llvm-cov", "clean", "--workspace", "--profraw-only"])
    run = command(["cargo", "llvm-cov", "run", "--profile", "cards", "--dep-coverage", "cantrip-cards",
                   "-p", "cantrip-script", "--bin", "corpus", "--no-report", "--", "--card", name])
    summary = [line for line in run.stdout.splitlines() if "scenarios:" in line]
    if not summary or not re.search(r"\b0 need cards\b", summary[-1]):
        raise RuntimeError(run.stdout[-4000:] + "\ncard-coverage: scenarios are missing dependencies or a summary")
    host = command(["rustc", "-vV"]).stdout
    triple = next((line.removeprefix("host: ") for line in host.splitlines() if line.startswith("host: ")), None)
    sysroot = command(["rustc", "--print", "sysroot"]).stdout.strip()
    extension = ".exe" if os.name == "nt" else ""
    tool_dir = Path(sysroot) / "lib" / "rustlib" / (triple or "") / "bin"
    llvm_profdata, llvm_cov = tool_dir / ("llvm-profdata" + extension), tool_dir / ("llvm-cov" + extension)
    raw = sorted(coverage_dir.glob(identity + "-*.profraw"))
    binary = coverage_dir / "cards" / ("corpus" + extension)
    if not raw or not llvm_profdata.is_file() or not llvm_cov.is_file() or not binary.is_file():
        raise RuntimeError("llvm-tools-preview or the instrumented corpus/profile is missing")
    with tempfile.TemporaryDirectory(prefix="card-coverage-") as scratch:
        profile, lcov = Path(scratch) / "card.profdata", Path(scratch) / "card.lcov"
        command([str(llvm_profdata), "merge", "-sparse", *map(str, raw), "-o", str(profile)])
        with lcov.open("w", encoding="utf-8") as output:
            command([str(llvm_cov), "export", "-format=lcov", f"-instr-profile={profile}", str(binary)],
                    stdout=output, stderr=subprocess.PIPE)
        return summary, lcov.read_text(encoding="utf-8")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    name = sys.argv[1]
    path = card_file(name)
    if path is None:
        print(f"{name}: no Rust file in {os.path.relpath(CARDS, ROOT)} (a card of core keywords only needs none).")
        return 0
    with build_lock(ROOT):
        try:
            summary, lcov = coverage_lcov(name)
        except RuntimeError as error:
            print(str(error))
            sys.exit("card-coverage: the coverage run failed (is cargo-llvm-cov installed?)")
        return report_coverage(name, path, summary, lcov)


def report_coverage(name, path, summary, lcov):
    target = os.path.normcase(os.path.abspath(path))
    hits, current = {}, None
    for line in lcov.splitlines():
        if line.startswith("SF:"):
            named = line[3:].strip()
            if named.startswith("\\\\?\\"):
                named = named[4:]  # llvm-cov writes Windows paths in their long form, \\?\D:\...
            current = os.path.normcase(os.path.abspath(named)) == target
        elif current and line.startswith("DA:"):
            number, count = line[3:].strip().split(",")[:2]
            hits[int(number)] = hits.get(int(number), 0) + int(count)
    source = open(path, encoding="utf-8").read().splitlines()
    exempt = const_fn_lines(source)
    hits = {n: c for n, c in hits.items() if n not in exempt}
    missed = sorted(n for n, c in hits.items() if c == 0)
    rel = os.path.relpath(path, ROOT)
    if summary:
        print(summary[-1].strip())
    if not hits:
        passed = re.search(r"(\d+) pass,", summary[-1]) if summary else None
        if passed and int(passed.group(1)) > 0 and not re.search(r"\bfn\s+\w+\s*\(", "\n".join(source)):
            print(f"{rel}: no executable card-specific Rust lines; the focused scenarios passed and exercise its declared abilities")
            return 0
        print(f"{rel}: no instrumented lines ran or exist - does any scenario hold {name}?")
        return 1
    if not missed:
        print(f"{rel}: every line ran ({len(hits)} instrumented lines).")
        return 0
    print(f"{rel}: {len(missed)} of {len(hits)} instrumented lines never ran:")
    for n in missed:
        print(f"  {n:4}  {source[n - 1].rstrip() if n <= len(source) else ''}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
