"""Durable run identity, input receipts, and handoffs for card-lane.py."""
import datetime
import hashlib
import io
import json
import os
import subprocess
import tarfile
import re
from pathlib import Path

SCHEMA = 1


def default_base(lane, root):
    return "upstream/main" if "upstream" in lane.git(root, "remote").stdout.splitlines() else "origin/main"


def fetch_base(lane, root, ref):
    if ref.startswith("-") or ":" in ref or ".." in ref:
        raise lane.Refused("base-ref must name one commit or remote branch")
    if "/" in ref and not ref.startswith("refs/"):
        remote, branch = ref.split("/", 1)
        if remote in lane.git(root, "remote").stdout.splitlines():
            lane.git(root, "fetch", "--quiet", remote, branch)
    tip = lane.ref_tip(root, ref + "^{commit}")
    if not tip:
        raise lane.Refused(f"no commit at {ref}")
    return tip


def manifest(lane, root, store, args):
    previous = store.read().get("run", {})
    ref = getattr(args, "base_ref", None) or previous.get("base_ref") or default_base(lane, root)
    if previous and ref != previous["base_ref"]:
        raise lane.Refused("a run's base-ref is immutable; use a new queue")
    latest = fetch_base(lane, root, ref)
    base = previous.get("base_sha") or latest
    repo = getattr(args, "repo", None) or previous.get("repo")
    if not repo:
        remote = ref.split("/", 1)[0]
        url = lane.git(root, "remote", "get-url", remote, check=False).stdout.strip().removesuffix(".git")
        github = re.search(r"github\.com[:/]([\w.-]+/[\w.-]+)$", url)
        repo = github[1] if github else ""
    if repo and not __import__("re").fullmatch(r"[\w.-]+/[\w.-]+", repo):
        raise lane.Refused("repo must be owner/repository")
    result = dict(previous, schema=SCHEMA, base_ref=ref, base_sha=base,
                  inventory_sha=latest, repo=repo, branch=store.branch,
                  delivery=getattr(args, "delivery", None) or previous.get("delivery", "pr"),
                  implementor_model=getattr(args, "implementor_model", None) or previous.get("implementor_model", "inherit"),
                  reviewer_model=getattr(args, "reviewer_model", None) or previous.get("reviewer_model", "inherit"),
                  concurrency=getattr(args, "concurrency", None) or previous.get("concurrency", 1),
                  engine=bool(getattr(args, "engine", False) or previous.get("engine")),
                  tool_sha=hashlib.sha256(Path(lane.__file__).read_bytes()).hexdigest())
    if result["concurrency"] < 1:
        raise lane.Refused("concurrency must be positive")
    if previous:
        for field in ("repo", "delivery", "implementor_model", "reviewer_model", "concurrency", "engine"):
            if result[field] != previous.get(field):
                raise lane.Refused(f"a run's {field} is immutable; use a new queue")
    return result


def names_at(lane, root, ref):
    """Inventory content from the exact Git tree, never the caller's checkout."""
    tree = lane.ref_tip(root, ref + ":content")
    if not tree:
        return set()
    cache = os.path.join(lane.common_dir(root), "card-lane", "inventory", tree + ".json")
    try:
        return set(lane.read_json(cache))
    except (OSError, ValueError):
        pass
    archive = subprocess.run(["git", "archive", ref, "content"], cwd=root, capture_output=True, check=True).stdout
    names = set()
    with tarfile.open(fileobj=io.BytesIO(archive)) as stream:
        for entry in stream:
            if not entry.isfile() or not __import__("re").fullmatch(r"content/[^/]+/cards/[^/]+\.json", entry.name):
                continue
            with stream.extractfile(entry) as source:
                data = json.load(source)
            def visit(value):
                if isinstance(value, dict):
                    if isinstance(value.get("name"), str):
                        names.add(lane.slug(value["name"]))
                    for item in value.values():
                        visit(item)
                elif isinstance(value, list):
                    for item in value:
                        visit(item)
            visit(data)
    os.makedirs(os.path.dirname(cache), exist_ok=True)
    lane.write_json(cache, sorted(names))
    return names


def reconcile(lane, state, known, sha):
    retired = []
    for card in state["cards"]:
        if card["status"] in ("pending", "deferred-pr") and not card.get("claim") and lane.slug(card["name"]) in known:
            card.update(status="done", finished=lane.now(), commit=None, claim=None,
                        upstream_sha=sha, note="already implemented on the canonical base")
            card.pop("upstream_prs", None)
            retired.append(card["name"])
    return retired


def source_names(lane, root, args, base):
    if getattr(args, "backlog", False):
        if args.format or args.card or args.names or (args.list and args.file):
            raise lane.Refused("--backlog is its own source; narrow it with one of --list or --file")
        if args.file:
            return sorted(lane.file_names(args.file), key=str.casefold)
        paths = lane.git(root, "ls-tree", "-r", "--name-only", base, "--", "docs/cube-lists").stdout.splitlines()
        paths = [p for p in paths if p.endswith(".txt") and (not args.list or p == f"docs/cube-lists/{args.list}.txt")]
        if args.list and not paths:
            raise lane.Refused(f"no cube list {args.list} on {base}")
        names = set()
        for path in paths:
            for line in lane.git(root, "show", f"{base}:{path}").stdout.splitlines():
                name = re.sub(r"^\d+x?\s+", "", line.strip())
                if name and not name.startswith("#"):
                    names.add(name)
        return sorted(names, key=str.casefold)
    if getattr(args, "list", None):
        raise lane.Refused("--list goes with --backlog")
    names = []
    if args.format:
        found = lane.git(root, "show", f"{base}:docs/formats/{args.format}.json", check=False)
        if found.returncode:
            raise lane.Refused(f"no format list {args.format} on the canonical base")
        names += [c["name"] for c in json.loads(found.stdout).get("cards", []) if c.get("name")]
    if args.file:
        names += lane.file_names(args.file)
    names += args.card + args.names
    if not names:
        raise lane.Refused("name the source: --format, --file, --backlog or --card")
    return list({name.casefold(): name for name in names}.values())


def fingerprint(lane, root, store, card):
    """Index blobs plus actual changed/untracked bytes and ignored evidence; HEAD alone is insufficient."""
    digest = hashlib.sha256()
    def add(label, value):
        digest.update(label.encode() + b"\0" + value + b"\0")
    add("index", lane.git(root, "ls-files", "--stage", "-z").stdout.encode())
    # Read actual bytes even for assume-unchanged/skip-worktree files; Git's stat cache is not a proof receipt.
    paths = lane.git(root, "ls-files", "-z").stdout.split("\0")
    paths += lane.git(root, "ls-files", "--others", "--exclude-standard", "-z").stdout.split("\0")
    for relative in sorted(set(paths) - {""}):
        if relative.startswith(lane.PACKETS + "/"):
            continue
        path = os.path.join(root, relative)
        add(relative, Path(path).read_bytes() if os.path.isfile(path) else b"<deleted>")
    packet = lane.packet_dir(root, store, card["name"])
    for filename in ("report.json", "reference.md", "content.json", "prior-art.md"):
        path = os.path.join(packet, filename)
        add(filename, Path(path).read_bytes() if os.path.isfile(path) else b"<missing>")
    for directory in sorted({lane.HERE, lane.tool_dir()}):
        for filename in sorted(os.listdir(directory)):
            if filename.endswith((".py", ".ps1")):
                add("tool/" + directory + "/" + filename, Path(directory, filename).read_bytes())
    for directory in (Path(root, ".cargo"), Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))):
        for filename in ("config", "config.toml"):
            path = directory / filename
            add(str(path), path.read_bytes() if path.is_file() else b"<missing>")
    config = lane.git(root, "config", "--get-regexp", r"^(core\.|filter\.)", check=False).stdout
    add("git-config", config.encode())
    relevant = {k: v for k, v in os.environ.items() if k.startswith(("CARGO", "RUST", "CANTRIP")) or k in ("PATH", "RUSTUP_TOOLCHAIN")}
    add("environment", json.dumps(relevant, sort_keys=True).encode())
    for program in ("cargo", "rustc"):
        result = lane.run([program, "--version"], root)
        add(program, (str(result.returncode) + result.stdout + result.stderr).encode())
    add("run", json.dumps(store.read().get("run", {}), sort_keys=True).encode())
    return digest.hexdigest()


def reusable(lane, root, store, card):
    try:
        receipt = lane.read_json(lane.check_file(root, store, card["name"]))
        age = datetime.datetime.now(datetime.timezone.utc) - datetime.datetime.fromisoformat(receipt["time"])
        required = {"scope", "oracle", "scenarios", "corpus", "lint", "citations", "report", "coverage", "mutation"}
        passed = {g["name"] for g in receipt.get("gates", []) if g.get("status") == "pass"}
        if (required <= passed and receipt.get("passed") and not receipt.get("fast") and receipt.get("schema") == SCHEMA
                and age.total_seconds() < 1800 and receipt.get("fingerprint") == fingerprint(lane, root, store, card)):
            return receipt["gates"]
    except (OSError, ValueError, KeyError):
        pass
    return None


def handoff_path(store, card):
    return os.path.join(store.dir, "handoffs", card + ".json")


def evidence(lane, root, store, card):
    packet = Path(lane.packet_dir(root, store, card["name"]))
    return {name: hashlib.sha256((packet / name).read_bytes()).hexdigest()
            for name in ("report.json", "reference.md", "content.json", "prior-art.md", "check.json")
            if (packet / name).is_file()}


def ready(lane, root, store):
    leases, members = lane.pool_claims(lane.common_dir(root))
    result = []
    for card in store.read()["cards"]:
        if card.get("review", {}).get("state") != "awaiting-review":
            continue
        packet = lane.read_json(handoff_path(store, lane.slug(card["name"])))
        lease = packet.get("lease")
        if lease and (lease not in leases or any(m.get("lease") == lease for m in members.values())):
            continue
        if not os.path.isdir(packet["worktree"]) or lane.status_paths(packet["worktree"]):
            continue
        if lane.head_sha(packet["worktree"]) != packet["sha"]:
            continue
        result.append(packet)
    return result
