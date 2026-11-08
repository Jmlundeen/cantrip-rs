"""Breaks a card's code one small change at a time and runs its scenarios against each: every change no scenario
catches is a decision of the card's own code that no expectation would catch being wrong (the add-cards skill's
"break the card", step 5, done by the machine).

    python tools/maintenance/card-mutate.py "Card Name"            # every change, one build and run each
    python tools/maintenance/card-mutate.py "Card Name" --list     # the changes it would try, nothing built
    python tools/maintenance/card-mutate.py "Card Name" --lines 12-30 --max 20
    python tools/maintenance/card-mutate.py "Card Name" --file crates/cantrip-mechanics/src/tokens.rs --lines 40-60
    python tools/maintenance/card-mutate.py "Card Name" --whole    # with every card in the build, not only those played
    python tools/maintenance/card-mutate.py "Card Name" --check    # fresh focused baseline, no mutations
    python tools/maintenance/card-mutate.py "Card Name" --prepare  # warm its checked mutate-profile binary
    python tools/maintenance/card-mutate.py "Card Name" --probes probes.json

The changes: a number one more and one less; `<` and `<=`, `>` and `>=` swapped, `==` and `!=`; `&&` and `||`;
`true` and `false`; a `!` dropped; `+` and `-`; a statement that calls the game left out. Each is built and the card's
scenarios run (`corpus -- --card`, under `CANTRIP_VERIFY` as the test suite runs them): one that fails is CAUGHT, one
that passes SURVIVED (a missing expectation - or an equivalent change, which the report names as such when you have read
it), one that does not compile is skipped. Exact source bytes are restored after every change and at exit, then the
restored baseline is run again.

A build holds only the cards the card's scenarios play (the `mutate` profile, with `CANTRIP_CARDS_ONLY` naming them, which
crates/cantrip-cards/build.rs reads): seconds a change, where a build of every card takes most of a minute. The profile keeps
its own place under `target/`, so it costs the `cards` build nothing; the first run in a tree builds the engine for it. A card
reached some way the scan cannot see is retried against the whole registry. Missing cards, build errors and missing
scenario reports never count as caught. Small-build panics are retried whole, since a missing definition can panic.

Probes are a JSON list (or {"probes": [...]}) of {"name", "file", "before", "after", "expected_failing_assertion"}.
Each unique exact replacement is applied alone to a repository-relative Rust source file. The expected assertion must
occur in a failure diagnostic before its trace, not merely in a printed action or state dump.
"""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

from card_tool_runtime import build_env, build_lock

sys.stdout.reconfigure(encoding="utf-8")

_spec = importlib.util.spec_from_file_location("new_card", os.path.join(os.path.dirname(__file__), "new-card.py"))
new_card = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(new_card)

ROOT = new_card.ROOT
PROFILE = "mutate"
FEW_CARDS = os.path.join(ROOT, "target", "card-mutate-cards.txt")

# (name, pattern, replacement): each match on a code line is one change.
SWAPS = [
    # A comparison has a space on each side; a generic (`Vec<ObjectId>`, `Result<(), Suspend> {`) does not.
    ("<= to <", r"(?<=\s)<=(?=\s)", "<"),
    ("< to <=", r"(?<=\s)<(?=\s)", "<="),
    (">= to >", r"(?<=\s)>=(?=\s)", ">"),
    ("> to >=", r"(?<=\s)>(?=\s)", ">="),
    ("== to !=", r"==", "!="),
    ("!= to ==", r"!=", "=="),
    ("&& to ||", r"&&", "||"),
    ("|| to &&", r"(?<=\s)\|\|(?=\s)", "&&"),
    ("true to false", r"\btrue\b", "false"),
    ("false to true", r"\bfalse\b", "true"),
    ("! dropped", r"(?<![\w!=)\]])!(?=[\w(])", ""),
    ("+ to -", r"(?<=\s)\+(?=\s)", "-"),
    ("- to +", r"(?<=\s)-(?=\s)", "+"),
]

NUMBER = re.compile(r"(?<![\w.#])(\d+)(?![\w.])")
# A statement that does something to the game: `g.draw(you, 1)?;`, `effects::pump(g, ...);`.
CALL_STATEMENT = re.compile(r"^(\s*)(?!let\b|return\b|Ok\(|Err\()((?:g|game)\.\w+|\w+::\w+(?:::\w+)*)\(.*\)\??;\s*$")


def code_spans(line):
    """The parts of a line outside string literals and comments: (start, end) pairs."""
    spans, start, i, quote = [], 0, 0, None
    while i < len(line):
        c = line[i]
        if quote:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                quote, start = None, i + 1
        elif c == '"':
            spans.append((start, i))
            quote = c
        elif line.startswith("//", i):
            spans.append((start, i))
            return spans
        i += 1
    if not quote:
        spans.append((start, len(line)))
    return spans


def mutants(lines, wanted):
    """Every change to the code lines in `wanted`: (line number, what, new line)."""
    out = []
    for n, line in enumerate(lines, 1):
        if n not in wanted:
            continue
        stripped = line.strip()
        if not stripped or stripped.startswith(("//", "use ", "#[", "pub static", "static ", "const ", "pub const")) \
                or "oracle::" in line:
            continue
        spans = code_spans(line)

        def inside(m):
            return any(a <= m.start() and m.end() <= b for a, b in spans)

        for m in NUMBER.finditer(line):
            if not inside(m) or re.search(r"(?:as\s+u\d+|\{|::)\s*$", line[:m.start()]):
                continue
            value = int(m.group(1))
            for new in (value + 1, value - 1):
                if new >= 0:
                    out.append((n, f"{value} to {new}", line[:m.start()] + str(new) + line[m.end():]))
        for what, pattern, replacement in SWAPS:
            for m in re.finditer(pattern, line):
                if inside(m) and "->" not in line[max(0, m.start() - 1):m.end() + 1] and "=>" not in line[max(0, m.start() - 1):m.end() + 1]:
                    out.append((n, what, line[:m.start()] + replacement + line[m.end():]))
        m = CALL_STATEMENT.match(line)
        if m and inside(re.search(re.escape(m.group(2)), line)):
            out.append((n, "statement left out", m.group(1) + "// (left out by card-mutate)"))
    return out


def parse_lines(text, count):
    if not text:
        return set(range(1, count + 1))
    wanted = set()
    for part in text.split(","):
        a, _, b = part.partition("-")
        wanted.update(range(int(a), int(b or a) + 1))
    return wanted


def deck_cards(text):
    """The cards a scenario's decks, sideboards and companions hold, as its parser reads them."""
    names, reading = [], False
    for line in text.split("[Actions]", 1)[0].splitlines():
        line = line.strip()
        if line.startswith("["):
            reading = line.upper().endswith((" DECK]", " SIDEBOARD]", " COMPANION]"))
        elif reading and line and not line.startswith("#"):
            names.append(re.sub(r"^\d+\s+|\s+[xX]\d+$", "", line).strip())
    return names


def imports(text):
    """The names a file's `use` lines bring in (an alias is the name), each with the path it stands for."""
    names = {}
    for path in re.findall(r"^\s*(?:pub\s+)?use\s+([^;]*);", text, re.M):
        head, brace, rest = path.partition("{")
        for item in (rest.rstrip("} \n").split(",") if brace else [path]):
            item = " ".join(item.split())
            if item:
                full, _, alias = (head * bool(brace) + item).partition(" as ")
                names[alias.strip() or full.split("::")[-1].strip()] = full.strip()
    return names


def shared_definitions():
    """A card file for each definition the mechanics crate shares, a token or a named emblem, which comes into the card table
    only in some card's `EXTRA`: without one the game stops at it, by a route no deck names."""
    providers = {}
    for name in sorted(os.listdir(new_card.CARDS)):
        with open(os.path.join(new_card.CARDS, name), encoding="utf-8") as f:
            text = f.read()
        found = re.search(r"pub static EXTRA:[^=]*=(.*?);\s*$", text, re.S | re.M)
        if not found:
            continue
        for shared in re.findall(r"&(\w+(?:::\w+)+)", found[1]):
            providers.setdefault(shared, name[:-3])
        # A bare entry the file does not define itself comes in by a `use`, an alias (`as`) included.
        imported = imports(text)
        for bare in re.findall(r"&(\w+)\s*[,\]]", found[1]):
            if bare in imported and not re.search(rf"\b(?:static|const)\s+{bare}\b", text):
                providers.setdefault(imported[bare], name[:-3])
    return set(providers.values())


def played(card):
    """The card files it takes to play this card's scenarios: their decks' cards, the files those name, and the definitions
    any of them may reach."""
    stems = {new_card.file_stem(card)}
    wanted = card.lower()
    for relative in new_card.scenario_files(card):
        with open(os.path.join(new_card.ROOT, relative), encoding="utf-8-sig") as f:
            decks = deck_cards(f.read())
        if wanted in (deck.lower() for deck in decks):
            stems.update(new_card.file_stem(deck) for deck in decks)
    exists = lambda stem: os.path.exists(os.path.join(new_card.CARDS, stem + ".rs"))
    stems = {stem for stem in stems if exists(stem)} | shared_definitions()
    todo = list(stems)
    while todo:
        with open(os.path.join(new_card.CARDS, todo.pop() + ".rs"), encoding="utf-8") as f:
            text = f.read()
        for stem in re.findall(r"\b(?:super|crate)::(?:r#)?(\w+)::", text):
            if stem not in stems and exists(stem):
                stems.add(stem)
                todo.append(stem)
    return stems


SUMMARY = re.compile(r"(\d+) scenarios: (\d+) pass, (\d+) fail, (\d+) known bugs, (\d+) need cards")


def classify_run(returncode, out):
    """An exit code alone is never proof that a mutant was caught."""
    summary = SUMMARY.search(out)
    if not summary:
        return "run error", out
    _, passed, failed, _, blocked = map(int, summary.groups())
    if blocked:
        return "cards missing", out
    if "does not parse:" in out:
        return "invalid scenarios", out
    if not passed and not failed:
        return "no scenarios", out
    if returncode == 0 and failed == 0:
        return "survived", out
    if returncode != 0 and failed and re.search(r"^  (?:FAIL|PANIC) ", out, re.M):
        return "caught", out
    return "run error", out


def corpus_env(few):
    env = build_env(ROOT)
    if few:
        env["CANTRIP_CARDS_ONLY"] = FEW_CARDS
    return env


def build_corpus(few=False):
    proc = subprocess.run(
        ["cargo", "build", "--profile", PROFILE, "-q", "-p", "cantrip-script", "--bin", "corpus"],
        cwd=ROOT, env=corpus_env(few), capture_output=True, text=True, encoding="utf-8", errors="replace")
    return proc.returncode, proc.stdout + proc.stderr


def run_scenarios(card, few=False):
    """Build and execution are separate, so any Cargo failure is unambiguously not caught."""
    code, build_output = build_corpus(few)
    if code:
        return "no build", build_output
    binary = Path(corpus_env(few)["CARGO_TARGET_DIR"]) / PROFILE / ("corpus.exe" if os.name == "nt" else "corpus")
    proc = subprocess.run([str(binary), "--card", card, "--brief"], cwd=ROOT, env=corpus_env(few),
                          capture_output=True, text=True, encoding="utf-8", errors="replace")
    verdict, out = classify_run(proc.returncode, proc.stdout + proc.stderr)
    return verdict, build_output + out


def safe_scenarios(card, few):
    """Validate hidden dependencies by replaying a partial-registry failure whole.

    Panic replay also covers indirect definitions that neither the deck scan nor
    the corpus's preflight missing-card list can see. Keep the whole registry for
    subsequent runs once the closure proves insufficient.
    """
    verdict, out = run_scenarios(card, few)
    if few and (verdict in ("cards missing", "no build") or re.search(r"^  PANIC ", out, re.M)):
        print(f"focused registry gave {verdict}; retrying with every card", flush=True)
        if verdict == "no build":
            print(out[-4000:], flush=True)
        verdict, whole_output = run_scenarios(card, False)
        out = whole_output
        few = False
    return verdict, out, few


def name_few_cards(card):
    """Writes the list only on change: an unchanged closure keeps Cargo artifacts warm."""
    text = "\n".join(sorted(played(card))) + "\n"
    path = Path(FEW_CARDS)
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_text(encoding="utf-8") != text:
        path.write_text(text, encoding="utf-8", newline="\n")


def source_path(relative):
    """Only actual Rust sources in this repository, never a traversal or escaping symlink."""
    relative = Path(relative)
    root = Path(ROOT).resolve()
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError(f"source path must be repository-relative: {relative}")
    path = (root / relative).resolve()
    if not path.is_relative_to(root / "crates") or path.suffix != ".rs" or not path.is_file():
        raise ValueError(f"not a Rust source under crates/: {relative}")
    return path


def load_probes(path):
    data = json.loads(Path(path).read_text(encoding="utf-8-sig"))
    if isinstance(data, dict):
        data = data.get("probes")
    if not isinstance(data, list) or not data:
        raise ValueError("probes must be a nonempty JSON list")
    changes, originals, names = [], {}, set()
    required = {"name", "file", "before", "after", "expected_failing_assertion"}
    for probe in data:
        if not isinstance(probe, dict) or set(probe) != required:
            raise ValueError(f"each probe needs exactly {', '.join(sorted(required))}")
        if any(not isinstance(probe[key], str) for key in required):
            raise ValueError("probe fields must all be strings")
        if not probe["name"].strip() or any(char in probe["name"] for char in "\r\n") \
                or probe["name"] in names or not probe["expected_failing_assertion"].strip():
            raise ValueError("probe names must be nonempty and unique; expected assertion must be nonempty")
        names.add(probe["name"])
        target = source_path(probe["file"])
        original = originals.setdefault(target, target.read_bytes())
        before, after = probe["before"].encode("utf-8"), probe["after"].encode("utf-8")
        if not before or before == after or original.count(before) != 1:
            raise ValueError(f"{probe['name']}: before must occur exactly once and after must change it")
        changes.append(dict(probe, path=target, changed=original.replace(before, after, 1)))
    return changes, originals


def failing_diagnostics(out):
    """Only failure causes, excluding action traces, state dumps and known-bug reports."""
    diagnostics = []
    for block in re.split(r"(?=^  (?:FAIL|PANIC|pass|known bug|FIXED) )", out, flags=re.M):
        if block.startswith("  FAIL "):
            diagnostics.append(re.split(r"\n(?:Outstanding:|Trace:|State:|\d+ scenarios:)", block, maxsplit=1)[0])
    return "\n".join(diagnostics)


def run_changes(card, changes, originals, few=True):
    """Fresh proof, one independent break at a time, exact restoration even on exceptions."""
    if few:
        name_few_cards(card)
    print("the unchanged card first: its scenarios must pass", flush=True)
    verdict, out, few = safe_scenarios(card, few)
    if verdict != "survived":
        print(out[-4000:])
        print(f"{card}'s scenarios do not pass as the card stands ({verdict}): fix that first")
        return 1
    results, survivors, incomplete = {"caught": 0, "survived": 0, "no build": 0}, [], False
    restored = "run error"
    started = time.monotonic()
    try:
        for i, change in enumerate(changes, 1):
            path = change["path"]
            if path.read_bytes() != originals[path]:
                raise RuntimeError(f"source changed during mutation: {path}")
            try:
                path.write_bytes(change["changed"])
                verdict, out, few = safe_scenarios(card, few)
            finally:
                path.write_bytes(originals[path])
            expected = change.get("expected_failing_assertion")
            if expected and verdict == "caught" and expected not in failing_diagnostics(out):
                verdict = "wrong assertion"
            results[verdict if verdict in results else "no build"] += 1
            if verdict == "survived":
                survivors.append(change)
            if verdict not in ("caught", "survived", "no build") or (expected and verdict != "caught"):
                incomplete = True
            rel = path.relative_to(Path(ROOT).resolve()).as_posix()
            print(f"[{i}/{len(changes)}] {rel}:{change.get('line', '-')} {change['name']}: "
                  f"{'SURVIVED' if verdict == 'survived' else verdict}", flush=True)
            if expected and verdict == "caught":
                evidence = [line for line in failing_diagnostics(out).splitlines() if expected in line]
                print("\n".join(evidence[:5]), flush=True)
                summary = SUMMARY.search(out)
                if summary:
                    print(summary.group(0), flush=True)
            elif expected or verdict not in ("caught", "survived"):
                print(out[-4000:], flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
        # Execute too: rebuilding alone did not prove that restoration handed over a passing baseline.
        print("the restored card: its scenarios must pass again", flush=True)
        restored, out, few = safe_scenarios(card, few)
        print(out[-4000:], flush=True)
        if restored != "survived":
            print(f"ERROR: restored baseline failed ({restored})", flush=True)
    print(f"\n{results['caught']} caught, {results['survived']} survived, {results['no build']} did not build"
          f" ({(time.monotonic() - started) / 60:.1f} min); source bytes restored")
    for change in survivors:
        rel = change["path"].relative_to(Path(ROOT).resolve()).as_posix()
        if "line" in change:
            print(f"SURVIVED {rel}:{change['line']}: {change['name']}\n"
                  f"    was: {change['before'].strip()}\n    now: {change['after'].strip()}")
        else:
            print(f"SURVIVED probe {change['name']}: {rel}")
    if survivors:
        print("Each survivor is an expectation missing - or a change that means the same thing (say so in the report).")
    return int(bool(survivors) or incomplete or restored != "survived")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("card")
    parser.add_argument("--file", help="the file to change, when not the card's own (a shared mechanic it uses)")
    parser.add_argument("--lines", help="only these lines, as 12-30,41")
    parser.add_argument("--max", type=int, default=0, help="at most this many changes")
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--list", action="store_true", help="list the changes and build nothing")
    modes.add_argument("--check", action="store_true", help="run a fresh focused baseline without changing source")
    modes.add_argument("--prepare", action="store_true", help="build the checked mutate-profile corpus without running it")
    modes.add_argument("--probes", metavar="JSON", help="run named exact semantic replacements")
    parser.add_argument("--whole", action="store_true", help="build every card")
    args = parser.parse_args()
    if args.probes and (args.file or args.lines or args.max):
        parser.error("--probes cannot be combined with --file, --lines or --max")
    with build_lock(ROOT):
        if args.check or args.prepare:
            few = not args.whole
            if few:
                name_few_cards(args.card)
            if args.prepare:
                code, out = build_corpus(few)
                if code and few:
                    print("focused build failed; preparing the whole registry instead", flush=True)
                    code, out = build_corpus(False)
                print(out, end="")
                if not code:
                    print(f"prepared {args.card}: profile {PROFILE}, private target {corpus_env(False)['CARGO_TARGET_DIR']}")
                return int(code != 0)
            verdict, out, few = safe_scenarios(args.card, few)
            print(out, end="")
            if verdict != "survived":
                print(f"card-mutate --check: {verdict}")
            return int(verdict != "survived")
        if args.probes:
            changes, originals = load_probes(args.probes)
        else:
            relative = args.file or os.path.relpath(os.path.join(new_card.CARDS, new_card.file_stem(args.card) + ".rs"), ROOT)
            path = source_path(relative)
            original = path.read_bytes()
            lines = original.decode("utf-8").splitlines(keepends=True)
            found = mutants(lines, parse_lines(args.lines, len(lines)))
            if args.max:
                found = found[:args.max]
            changes = []
            for n, what, new in found:
                # Statement removal does not carry its original line ending.
                ending = "\r\n" if lines[n - 1].endswith("\r\n") else "\n" if lines[n - 1].endswith("\n") else ""
                if ending and not new.endswith(ending):
                    new += ending
                changed = "".join(lines[:n - 1] + [new] + lines[n:]).encode("utf-8")
                changes.append({"name": what, "line": n, "path": path, "changed": changed,
                                "before": lines[n - 1], "after": new})
            originals = {path: original}
        if args.list:
            for change in changes:
                print(f"{change['path'].relative_to(Path(ROOT).resolve()).as_posix()}:{change['line']}: {change['name']}\n"
                      f"    {change['after'].strip()}")
            print(f"{len(changes)} changes")
            return 0
        return run_changes(args.card, changes, originals, not args.whole)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, TimeoutError) as error:
        sys.exit(f"card-mutate: {error}")
