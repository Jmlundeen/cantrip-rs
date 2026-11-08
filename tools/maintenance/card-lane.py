"""The card lane: one shared queue for a pool of implementors, and one packet per card.

    python tools/maintenance/card-lane.py queue  [--queue NAME] [--format F | --file PATH | --card NAME ...]
    python tools/maintenance/card-lane.py next   --id WORKER [--queue NAME] [--card NAME] [--refresh] [--engine] [--issue N]
    python tools/maintenance/card-lane.py status [--queue NAME] [--id WORKER]
    python tools/maintenance/card-lane.py check  [--queue NAME] [--id WORKER | --card NAME] [--fast]
    python tools/maintenance/card-lane.py commit --id WORKER [--queue NAME] [--issue N] [--note TEXT]
    python tools/maintenance/card-lane.py done   --id WORKER [--queue NAME] [--commit SHA | --no-commit] [--note TEXT]
    python tools/maintenance/card-lane.py block  --id WORKER --rule CR [--queue NAME] [--note TEXT]
    python tools/maintenance/card-lane.py release --id WORKER --reason TEXT [--queue NAME]
    python tools/maintenance/card-lane.py abandon --id WORKER --reason TEXT [--queue NAME]
    python tools/maintenance/card-lane.py share  --id WORKER [--queue NAME]

The lane is dispatch, not implementation: a worker still follows `.claude/skills/add-cards/SKILL.md` for the card.
It moves the lane's queue and packets into the Cantrip checkout's own worktree pool, so several implementors work in
parallel without taking the same card:

- **One shared queue.** The state is `.git/card-lane/<queue>/queue.json`, beside the pool's own state and shared by
  every worktree of the repository (never committed). A claim is made under a lock, so two `next` calls can never take
  the same card.
- **An id owns a card.** `next --id w1` opens the next pending card and records w1 as its claimant. Running it again
  with the same id prints the same card's packet again; it does not take another card. A second id gets the next card
  (and a second worktree is required: one worktree holds one open card).
- **Packets live in the implementor's worktree**, under `.card-lane/<queue>/<card>/` (gitignored, regenerable, thrown
  away with the slot). A packet holds `brief.md`, `reference.md` (everything `new-card.py` prints: Oracle text,
  rulings, CR, prior art, test points), `prior-art.md` (the matched cards' code and a scenario), the content JSON and
  `state.json`. Nothing is written outside the worktree.
- **One shared branch.** Every claim starts from `card-lane/<queue>`: `next` merges its tip into the slot before work,
  and `share` (or `done`, `block`, `release`) updates it from the slot with a compare-and-swap. A `Mechanics:` or
  `Engine:` commit made by one implementor is therefore in the next implementor's tree, so shared mechanics and
  implemented gaps are reused instead of written twice.
- **The pool is the context.** In a pool slot the lane reads `.git/cantrip-worktrees.json` (through
  `tools/maintenance/worktrees.py`, the pool's own model): the slot must be leased and have a joined member, and the
  claim records that lease and member. A worktree outside the pool may still be used solo, and its claim says so.

There is no agent loop, no driver and no generated track: a worker (or its coordinator) runs the commands.

The card's quality gates are the lane's too, adapted from the add-cards flow and run where the worker can see them:

- **`check`** runs the claim and packet check, the path scope (the card's content, its Rust file, its scenarios, a
  shared mechanic, a client change, or the core only for a card opened `--engine`), Scryfall's current Oracle text,
  the scenario shape (no TODO or known bug, an `[Expect]`, the cost paid exactly), `corpus --card`, `card-lint` and
  `card-dupes` (each LOOK answered in `report.json`), `cr.py check --show`, and the report (every printed line, each
  test point with its scenario line, each ruling). A full check adds `card-coverage` and `card-mutate`; `--fast`
  skips only those two while the card is being written.
- **`commit`** runs every gate, then writes the `Engine:`, `Mechanics:` and `Client:` commits the change needs, each
  with only its own paths, and exactly one `Card: <Name>` commit ending `Closes #N`; no commit carries
  `Co-Authored-By`. It closes the claim (`done`) and shares the branch. Nothing is committed if the gate refuses.
"""

import argparse
import contextlib
import datetime
import json
import os
import re
import shutil
import subprocess
import sys
import time
import unicodedata

sys.dont_write_bytecode = True

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import worktrees  # noqa: E402  the pool's state is read through its own model, never by editing its file

PACKETS = ".card-lane"
DEFAULT_QUEUE = "default"
LOCK_WAIT = 30.0
LOCK_STALE = 300.0  # a lock file this old was left by a killed command, and is broken
LOG_KEEP = 500
PRIOR_ART, PRIOR_CODE_CHARS, PRIOR_SCENARIO_CHARS = 3, 2500, 1800
ZERO = "0" * 40


class Refused(Exception):
    """The lane says no, and its message says why and what to do instead. Exit status 2."""


class Failed(Exception):
    """A command genuinely failed (git, new-card.py, the filesystem). Exit status 1."""


# ---- running --------------------------------------------------------------------------------------------------------

def run(args, cwd, check=False, env=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, encoding="utf-8", errors="replace",
                            env=env)
    if check and result.returncode != 0:
        raise Failed(f"{' '.join(str(a) for a in args)} failed in {cwd}:\n{(result.stderr or result.stdout).strip()}")
    return result


def git(root, *args, check=True):
    return run(["git", *args], root, check=check)


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")


def norm(path):
    return os.path.normcase(os.path.normpath(os.path.abspath(path)))


def worktree_root():
    """The checkout to work in: the caller's worktree (a pool slot, or the main checkout for a solo run)."""
    found = run(["git", "rev-parse", "--show-toplevel"], os.getcwd())
    if found.returncode != 0:
        raise Refused("run the card lane from inside the Cantrip checkout or one of its worktrees")
    return os.path.normpath(found.stdout.strip())


def common_dir(root):
    found = git(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
    return os.path.normpath(found.stdout.strip())


def current_branch(root):
    found = git(root, "symbolic-ref", "--quiet", "--short", "HEAD", check=False)
    return found.stdout.strip() or None


def require_branch(root):
    branch = current_branch(root)
    if not branch:
        raise Refused(f"{root} is on a detached HEAD: the lane needs a branch it can commit to and share")
    return branch


def status_paths(root):
    """The worktree's uncommitted changes, leaving out the lane's own packets."""
    lines = []
    for line in git(root, "status", "--porcelain").stdout.splitlines():
        path = line[3:].strip()
        if path.startswith(PACKETS + "/") or path == PACKETS:
            continue
        lines.append(line)
    return lines


def require_clean(root, why):
    dirty = status_paths(root)
    if dirty:
        raise Refused(f"{root} has uncommitted changes {why}:\n  " + "\n  ".join(dirty[:12]))


def head_sha(root):
    return git(root, "rev-parse", "HEAD").stdout.strip()


# ---- the queue ------------------------------------------------------------------------------------------------------

def slug(name):
    """The content slug the loader requires: the same fold `missing-cards.py` uses."""
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    folded = folded.lower().replace("'", "").replace("\u2019", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", folded)).strip("-")


def queue_name(value):
    name = str(value or DEFAULT_QUEUE).strip()
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,40}", name) or ".." in name:
        raise Refused(f"a queue name is letters, digits, '.', '_' or '-' (up to 41), not {value!r}")
    return name


def worker_id(value):
    wid = str(value or "").strip()
    if not wid:
        raise Refused("--id names this worker: the same id gets its open card back, and another id gets another card")
    if len(wid) > 80 or any(c.isspace() for c in wid):
        raise Refused("--id is one word, up to 80 characters")
    return wid


class Store:
    """One queue, in the repository's git common directory, shared by every worktree and changed under a lock."""

    def __init__(self, common, name):
        self.name = queue_name(name)
        self.dir = os.path.join(common, "card-lane", self.name)
        self.path = os.path.join(self.dir, "queue.json")
        self.lock_path = os.path.join(self.dir, "queue.lock")
        self.branch = f"card-lane/{self.name}"
        os.makedirs(self.dir, exist_ok=True)

    def read(self):
        for attempt in range(20):
            try:
                with open(self.path, encoding="utf-8") as f:
                    state = json.load(f)
                break
            except FileNotFoundError:
                state = {}
                break
            except (PermissionError, json.JSONDecodeError):
                if attempt == 19:
                    raise
                time.sleep(0.05)
        state.setdefault("queue", self.name)
        state.setdefault("branch", self.branch)
        state.setdefault("created", now())
        state.setdefault("updated", now())
        state.setdefault("cards", [])
        state.setdefault("log", [])
        return state

    def write(self, state):
        state["updated"] = now()
        state["log"] = state["log"][-LOG_KEEP:]
        temp = f"{self.path}.{os.getpid()}.tmp"
        with open(temp, "w", encoding="utf-8") as f:
            json.dump(state, f, indent=1, ensure_ascii=False)
        for attempt in range(50):  # Windows refuses to replace a file another process has open for a moment
            try:
                os.replace(temp, self.path)
                return
            except PermissionError:
                if attempt == 49:
                    raise
                time.sleep(0.05)

    @contextlib.contextmanager
    def locked(self):
        deadline = time.monotonic() + LOCK_WAIT
        while True:
            try:
                fd = os.open(self.lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
                os.write(fd, f"{os.getpid()} {now()}".encode())
                os.close(fd)
                break
            except FileExistsError:
                try:
                    if time.time() - os.path.getmtime(self.lock_path) > LOCK_STALE:
                        os.remove(self.lock_path)
                        continue
                except FileNotFoundError:
                    continue
                if time.monotonic() > deadline:
                    raise Refused(f"the queue's lock {self.lock_path} has been held for {LOCK_WAIT:.0f}s; try again")
                time.sleep(0.1)
        try:
            state = self.read()
            yield state
            self.write(state)
        finally:
            try:
                os.remove(self.lock_path)
            except FileNotFoundError:
                pass

    def log(self, state, event, **fields):
        state["log"].append({"time": now(), "event": event, **fields})


def card_of(state, name):
    for card in state["cards"]:
        if card["name"].casefold() == name.casefold():
            return card
    return None


def open_claim(state, wid):
    for card in state["cards"]:
        if card.get("status") == "working" and (card.get("claim") or {}).get("id") == wid:
            return card
    return None


def open_in(state, path):
    for card in state["cards"]:
        if card.get("status") == "working" and norm((card.get("claim") or {}).get("worktree", "")) == norm(path):
            return card
    return None


def implemented_names(root):
    """Every name and face name a content file already defines, by slug (as `missing-cards.py` reads them)."""
    known = set()
    content = os.path.join(root, "content")
    for dataset in sorted(os.listdir(content)) if os.path.isdir(content) else []:
        cards = os.path.join(content, dataset, "cards")
        if not os.path.isdir(cards):
            continue
        for entry in sorted(os.listdir(cards)):
            if not entry.endswith(".json"):
                continue
            known.add(slug(entry[: -len(".json")]))
            try:
                with open(os.path.join(cards, entry), encoding="utf-8") as f:
                    text = f.read()
            except OSError:
                continue
            for name in re.findall(r'"name"\s*:\s*"((?:[^"\\]|\\.)*)"', text):
                known.add(slug(name))
    return known


def format_names(root, fmt):
    path = os.path.join(root, "docs", "formats", f"{fmt}.json")
    if not os.path.isfile(path):
        raise Refused(f"no format list at docs/formats/{fmt}.json (the lists are the files in docs/formats)")
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return [entry.get("name") for entry in data.get("cards") or [] if entry.get("name")]


def file_names(path):
    if not os.path.isfile(path):
        raise Refused(f"no card list at {path}")
    names = []
    with open(path, encoding="utf-8-sig") as f:
        for line in f:
            name = re.sub(r"^\d+x?\s+", "", line.strip())
            if name and not name.startswith("#"):
                names.append(name)
    return names


def requested_names(args, root):
    names = []
    if getattr(args, "format", None):
        names += format_names(root, args.format)
    if getattr(args, "file", None):
        names += file_names(args.file)
    names += list(getattr(args, "card", None) or [])
    names += list(getattr(args, "names", None) or [])
    seen, unique = set(), []
    for name in names:
        key = name.casefold()
        if key not in seen:
            seen.add(key)
            unique.append(name)
    if not unique:
        raise Refused("name the cards: --format <format>, --file <list>, --card <name>, or positional names")
    return unique


# ---- the shared branch ----------------------------------------------------------------------------------------------

def branch_tip(root, branch):
    found = git(root, "rev-parse", "--verify", "--quiet", f"refs/heads/{branch}", check=False)
    return found.stdout.strip() if found.returncode == 0 else None


def ancestor(root, older, newer):
    return git(root, "merge-base", "--is-ancestor", older, newer, check=False).returncode == 0


def ensure_branch(root, store):
    """The queue's branch, created on origin/main the first time and then shared by every worktree."""
    tip = branch_tip(root, store.branch)
    if tip:
        return tip
    base = branch_tip(root, "origin/main") or head_sha(root)
    git(root, "branch", store.branch, base, check=False)  # another first worker may have won the race
    return branch_tip(root, store.branch) or base


def checked_out_elsewhere(root, branch):
    for block in git(root, "worktree", "list", "--porcelain").stdout.split("\n\n"):
        fields = dict(line.split(" ", 1) for line in block.splitlines() if " " in line)
        if fields.get("branch") == f"refs/heads/{branch}" and norm(fields.get("worktree", "")) != norm(root):
            return fields["worktree"]
    return None


def merge_branch(root, store):
    """Bring the queue's branch into this worktree before a card is opened."""
    tip = branch_tip(root, store.branch)
    if not tip or ancestor(root, tip, "HEAD"):
        return None
    require_clean(root, "before the shared branch is merged in")
    result = git(root, "merge", "--no-edit", tip, check=False)
    if result.returncode != 0:
        git(root, "merge", "--abort", check=False)
        raise Refused(f"{store.branch} does not merge cleanly into this worktree; nothing is left half-merged:\n"
                      f"{(result.stdout + result.stderr).strip()[-1200:]}\n"
                      f"Resolve it by hand with `git merge {store.branch}`, or tell the coordinator.")
    return tip


def publish(root, store):
    """Move the queue's branch to this worktree's HEAD, merging in whatever other workers shared meanwhile."""
    branch = require_branch(root)
    elsewhere = checked_out_elsewhere(root, store.branch)
    if elsewhere:
        raise Refused(f"{store.branch} is checked out in {elsewhere}: a shared branch is never checked out; "
                      f"work on the slot's own branch")
    shared = []
    for _ in range(5):
        old = branch_tip(root, store.branch) or ZERO
        new = head_sha(root)
        if old == new:
            return new, shared
        if old != ZERO and not ancestor(root, old, new):
            require_clean(root, f"before {store.branch} is merged in so it can be shared")
            result = git(root, "merge", "--no-edit", old, check=False)
            if result.returncode != 0:
                git(root, "merge", "--abort", check=False)
                raise Refused(f"{store.branch} moved on and does not merge cleanly with {branch}; nothing is left "
                              f"half-merged:\n{(result.stdout + result.stderr).strip()[-1200:]}\n"
                              f"Run `git merge {store.branch}`, resolve it, commit, and try again.")
            new = head_sha(root)
        result = git(root, "update-ref", f"refs/heads/{store.branch}", new, old, check=False)
        if result.returncode == 0:
            shared += git(root, "log", "--format=%h %s", f"{old}..{new}" if old != ZERO else f"-5 {new}").stdout.splitlines()
            return new, shared
        time.sleep(0.1)  # another worker updated the branch first: merge what it shared and retry
    raise Refused(f"{store.branch} kept moving while this worktree tried to share; try again")


# ---- the pool -------------------------------------------------------------------------------------------------------

def pool_context(root):
    """The lease and member of the pool slot this worktree is, read from the pool's own state.

    A worktree outside `.claude/worktrees` is a solo checkout: it may use the lane, and its claim says so.
    """
    common = common_dir(root)
    main = os.path.dirname(common)
    pool_dir = os.path.join(main, worktrees.POOL)
    here = norm(root)
    if here != norm(pool_dir) and not here.startswith(norm(pool_dir) + os.sep):
        return {"slot": None, "lease": None, "member": None, "agent": None}
    slot = os.path.basename(here)
    if not re.fullmatch(r"pool-\d+", slot):
        raise Refused(f"{root} is under .claude/worktrees but is not a pool slot; work only in a pool-<n> slot")
    pool = worktrees.Pool(root=main)
    state = pool.read()
    lease_id, lease = pool.lease_of(state, slot)
    if not lease:
        raise Refused(f"{slot} has no lease: the pool hands a slot out with `tools\\worktree.cmd acquire`, and "
                      f"a slot nobody holds is not worked in")
    member_id, member = pool.member_of(state, lease_id)
    if not member:
        raise Refused(f"{slot} is leased but has no member: the implementor joins it first (`tools\\worktree.cmd "
                      f"join <token> --agent-id <id>`), and only its one member may claim a card")
    if member.get("read_only"):
        raise Refused(f"{slot}'s member is read-only (a reviewer's token): it may inspect a card's scenarios but it "
                      f"does not claim, share or close one")
    return {"slot": slot, "lease": lease_id, "member": member_id, "agent": member.get("agent"),
            "purpose": lease.get("purpose", "")}


def pool_claims(common):
    """The pool's leases and members by id, so `status` can say whether a claim's slot is still there."""
    root = os.path.dirname(common)
    try:
        state = worktrees.Pool(root=root).read()
    except Exception:
        return {}, {}
    return state.get("leases") or {}, state.get("members") or {}


# ---- packets --------------------------------------------------------------------------------------------------------

def packet_dir(root, store, name):
    return os.path.join(root, PACKETS, store.name, slug(name))


def section(text, heading):
    """The lines of a `=== Heading` block of new-card.py's output, up to the next one."""
    lines, inside = [], False
    for line in text.splitlines():
        if line.startswith("=== "):
            if inside:
                break
            inside = line.startswith(heading)
            continue
        if inside:
            lines.append(line)
    return "\n".join(lines).strip()


def content_from(reference, root):
    match = re.search(r"^=== .+?\s+\((.+?)\)\s+oracle::", reference, re.M)
    if not match:
        return None
    path = os.path.join(root, match.group(1).replace("/", os.sep))
    return path if os.path.isfile(path) else None


def card_file_from(reference, root):
    for pattern in (r"^=== wrote (\S+\.rs) \(", r"^=== (\S+\.rs) exists:$"):
        match = re.search(pattern, reference, re.M)
        if match:
            path = os.path.join(root, match.group(1).replace("/", os.sep))
            return path if os.path.isfile(path) else None
    return None


def scenarios_from(reference):
    block = section(reference, "=== Scenarios naming it")
    return [line.strip() for line in block.splitlines() if line.strip().startswith("scenarios/")]


def prior_art(root, reference):
    """The matched implemented cards, each with its code and a scenario - the shape the new card copies."""
    stems = []
    for match in re.finditer(r"crates[/\\]cantrip-cards[/\\]src[/\\]cards[/\\](\w+)\.rs", reference):
        if match.group(1) not in stems:
            stems.append(match.group(1))
    parts = []
    for stem in stems[:PRIOR_ART]:
        card_path = os.path.join(root, "crates", "cantrip-cards", "src", "cards", f"{stem}.rs")
        scenarios = sorted(
            (os.path.join(root, "scenarios", entry) for entry in os.listdir(os.path.join(root, "scenarios"))),
            key=os.path.getsize,
        ) if os.path.isdir(os.path.join(root, "scenarios")) else []
        scenarios = [p for p in scenarios
                     if os.path.basename(p).startswith(stem.replace("_", "-")) and p.endswith(".cantrip")]
        if not os.path.isfile(card_path) and not scenarios:
            continue
        if os.path.isfile(card_path):
            with open(card_path, encoding="utf-8") as f:
                code = f.read()
            cut = "\n// ...cut here; the rest: crates/cantrip-cards/src/cards/" + os.path.basename(card_path) \
                if len(code) > PRIOR_CODE_CHARS else ""
            parts += [f"`crates/cantrip-cards/src/cards/{stem}.rs`", "", "```rust",
                      code[:PRIOR_CODE_CHARS].rstrip() + cut, "```", ""]
        if scenarios:
            with open(scenarios[0], encoding="utf-8") as f:
                text = f.read()[:PRIOR_SCENARIO_CHARS].rstrip()
            parts += [f"`scenarios/{os.path.basename(scenarios[0])}`", "", "```", text, "```", ""]
    if not parts:
        return ("No implemented card is matched for this card. Read the nearest abilities in `reference.md`, then search "
                "`crates/cantrip-api/src/game.rs` (the verbs) and `crates/cantrip-mechanics/src` (the shared mechanics) "
                "for each line marked `new`.\n")
    return "\n".join(parts).rstrip() + "\n"


def clean_relative(root, path):
    return os.path.relpath(path, root).replace(os.sep, "/") if path else None


def write_brief(root, store, card, claim, reference, packet):
    content = content_from(reference, root)
    card_file = card_file_from(reference, root)
    scenarios = scenarios_from(reference)
    member = claim.get("member")
    agent = claim.get("agent")
    address = (f"pool slot {claim['slot']} (lease {claim['lease']}, member {member}, agent {agent})" if claim.get("slot")
               else "a solo worktree (not a pool slot)")
    lines = [
        f"# Card: {card['name']}",
        "",
        f"Queue `{store.name}` - worker `{claim['id']}` - {address}.",
        f"Worktree: `{root}`",
        f"Packet: `{packet}`",
        "",
        "## What is here",
        "",
        "Read `reference.md` and `prior-art.md` first: the Oracle text, the official rulings, the CR rules, every prior",
        "card `new-card.py` matched (with its code and a scenario), and the test points. Then read the sections of",
        "`.claude/skills/add-cards/SKILL.md` this card needs - \"The loop for one card\" - and",
        "`.claude/skills/write-game-script/SKILL.md` for its scenarios. Read nothing else whole.",
        "",
        f"`{REPORT}` is yours to fill in as you go: every printed Oracle line and the code that implements it, each",
        "test point with the scenario line that catches it, each ruling (a scenario line, or why the card does not",
        "decide it), and a line answering every LOOK the lint prints. For a mutation survivor no scenario catches,",
        "add `{\"mutant\": \"<file>:<line>: <what>\", \"why\": \"<why it means the same>\"}` to `equivalent_mutants`.",
        "It is in the packet; it is never committed.",
        "",
        "## Goal",
        "",
        "The card is exact to its Oracle text and the Comprehensive Rules, every line of its code is executed by its",
        f"scenarios and passes, and it is committed as `Card: {card['name']}`, ending `Closes #N` for its issue.",
        "`commit` is the gated path: it runs every check, writes the `Engine:`, `Mechanics:` and `Client:` commits the",
        "change needs (each with only its own paths), then exactly one `Card:` commit, and closes the claim.",
        "",
        f"    python tools/maintenance/card-lane.py check --id {claim['id']} --queue {store.name} --fast",
        f"    python tools/maintenance/card-lane.py commit --id {claim['id']} --queue {store.name}",
        "",
        "`check --fast` skips only coverage and mutation while the card is being written; `commit` runs both. A failed",
        "check names the gate and nothing is committed.",
        "",
        (f"The issue is #{claim['issue']}; the Card commit ends `Closes #{claim['issue']}`." if claim.get("issue")
         else "No issue number is set: pass `--issue N` to `next` or `commit`, or set report.json's issue."),
        "",
        ("This card was opened with `--engine`: the core (cantrip-state, cantrip-rules, cantrip-api) may be changed,"
         " in its own `Engine:` commit." if claim.get("engine") else
         "The core is closed to this card: a missing rule is a gap, never a workaround."),
        "",
        "Claim the card's GitHub issue before working on it, as the skill says (`python tools/maintenance/card-issues.py",
        f"claim \"{card['name']}\"`), unless you already hold it.",
        "",
        "## Paths",
        "",
        f"- Card file: `{clean_relative(root, card_file) or 'none - the content file is the whole card'}`",
        f"- Content: `{clean_relative(root, content) or 'not written by new-card.py yet; the skill says how'}`",
        "- Existing scenarios: " + (", ".join(f"`{entry}`" for entry in scenarios) if scenarios else "none yet"),
        "- Scenario directory: `scenarios/`",
        f"- Shared branch: `{store.branch}` (merged into this worktree before the card was opened)",
        "",
        "## When the card needs the engine",
        "",
        (("This card may change the core: commit the core alone in its own `Engine:` commit. A rule the engine cannot"
          " express is still a gap: block it, never work around it.") if claim.get("engine") else
         ("The core is closed: a missing rule is a gap, never a workaround. Commit what is safe, log the gap on the"
          " issue, and close the claim with the rule:")),
        "",
        f"    python tools/maintenance/card-lane.py block --id {claim['id']} --queue {store.name} --rule <CR> --note \"...\"",
        "",
        "If the work has to stop instead, put the card back in the queue with",
        f"`release --id {claim['id']} --queue {store.name} --reason \"...\"`.",
        "",
        "## Sharing mechanics as you go",
        "",
        "After a `Mechanics:` or `Engine:` commit, and before the card is finished, run",
        "",
        f"    python tools/maintenance/card-lane.py share --id {claim['id']} --queue {store.name}",
        "",
        "The next implementor's `next` merges it, so a shared function is written once. `commit` and `done` share",
        "automatically.",
    ]
    if claim.get("slot"):
        lines += ["", "## Leaving the slot",
                  "",
                  "Commit everything first; the slot is reset when its lease is released. Your last command is `leave`",
                  "with the MEMBER id `join` printed: `tools\\worktree.cmd leave <MEMBER>`. Do not `git push`, and do not",
                  "release the lease - that is the coordinator's."]
    else:
        lines += ["", "## Solo",
                  "",
                  "This worktree is not a pool slot, so there is no member to leave. Commit the card on this branch;",
                  "`commit` (or `done`) shares it to the queue's branch."]
    return "\n".join(lines).rstrip() + "\n"


def build_packet(root, store, card, claim):
    """Everything the worker starts from, in the implementor's own worktree. Regenerable, never committed."""
    packet = packet_dir(root, store, card["name"])
    os.makedirs(packet, exist_ok=True)
    tool = os.path.join(root, "tools", "maintenance", "new-card.py")
    if os.path.isfile(tool):
        result = run([sys.executable, tool, card["name"]], root)
        reference = (result.stdout or "")
        if result.stderr.strip():
            reference += "\n[stderr]\n" + result.stderr
        if result.returncode != 0:
            reference += f"\n[new-card.py exited {result.returncode}]\n"
    else:
        reference = f"new-card.py is missing at {tool}\n"
    with open(os.path.join(packet, "reference.md"), "w", encoding="utf-8", newline="\n") as f:
        f.write(reference)
    content = content_from(reference, root)
    if content:
        shutil.copyfile(content, os.path.join(packet, "content.json"))
    report = os.path.join(packet, REPORT)
    if not os.path.isfile(report):
        write_json(report, report_skeleton(reference, content, (claim or {}).get("issue")))
    else:
        try:
            existing = read_json(report)
            if isinstance(existing, dict) and not existing.get("issue") and (claim or {}).get("issue"):
                existing["issue"] = int(claim["issue"])
                write_json(report, existing)
        except (OSError, json.JSONDecodeError, TypeError, ValueError):
            pass  # the check gate reports a report that cannot be read or filled
    with open(os.path.join(packet, "prior-art.md"), "w", encoding="utf-8", newline="\n") as f:
        f.write(prior_art(root, reference))
    with open(os.path.join(packet, "state.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump({"queue": store.name, "branch": store.branch, "card": card["name"], "claim": claim},
                  f, indent=1, ensure_ascii=False)
    brief = write_brief(root, store, card, claim, reference, packet)
    with open(os.path.join(packet, "brief.md"), "w", encoding="utf-8", newline="\n") as f:
        f.write(brief)
    return packet, brief


def save_patch(root, store, name):
    """Whatever the card still has uncommitted, kept in its packet before the claim is closed."""
    packet = packet_dir(root, store, name)
    os.makedirs(packet, exist_ok=True)
    diff = git(root, "diff", "HEAD", check=False).stdout
    if diff.strip():
        with open(os.path.join(packet, "work.patch"), "w", encoding="utf-8", newline="\n") as f:
            f.write(diff)
    untracked = [line[3:].strip() for line in git(root, "status", "--porcelain").stdout.splitlines()
                 if line.startswith("??") and not line[3:].strip().startswith(PACKETS + "/")]
    return packet if diff.strip() else None, untracked


# ---- the gates ------------------------------------------------------------------------------------------------------

REPORT = "report.json"
CHECK = "check.json"
ENGINE_DIRS = ("crates/cantrip-state/", "crates/cantrip-rules/", "crates/cantrip-api/")
ORACLE_DIRS = ("crates/cantrip-oracle/",)
HARNESS_DIRS = ("crates/cantrip-script/",)
HARNESS_DOCS = {"scenarios/README.md"}
MECHANICS_DIRS = ("crates/cantrip-mechanics/",)
CLIENT_DIRS = ("dotnet/", "web_src/", "crates/cantrip-host/", "crates/cantrip-ai-host/")
CLIENT_DOCS = {"docs/client-boundary.md", "docs/host-protocol.md", "docs/web-client.md",
               "docs/scenarios-in-the-browser.md"}
KNOWN_ISSUES = "docs/known-issues.md"


class GateFailed(Exception):
    """A gate says the card is not ready; the message is the line, `detail` the evidence."""

    def __init__(self, message, detail=""):
        super().__init__(message)
        self.detail = detail or ""


def read_text(path):
    with open(path, encoding="utf-8-sig") as f:
        return f.read()


def read_json(path):
    with open(path, encoding="utf-8-sig") as f:
        return json.load(f)


def write_json(path, data):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(data, f, indent=1, ensure_ascii=False)
        f.write("\n")


def changed_paths(root):
    """Every uncommitted path, with its two-letter status, in git's NUL form (names keep their spaces)."""
    out = git(root, "status", "--porcelain", "-z", "--untracked-files=all").stdout
    fields = out.split("\0")
    entries, i = [], 0
    while i < len(fields):
        entry = fields[i]
        i += 1
        if len(entry) < 4:
            continue
        status, path = entry[:2], entry[3:].replace("\\", "/")
        renamed = status[0] in "RC"
        if renamed and i < len(fields):
            i += 1  # the name it was
        if path == PACKETS or path.startswith(PACKETS + "/"):
            continue
        entries.append({"status": status, "path": path})
    return entries


def tree_sha(root):
    return git(root, "rev-parse", "HEAD^{tree}").stdout.strip()


def under(path, prefixes):
    return any(path == prefix.rstrip("/") or path.startswith(prefix) for prefix in prefixes)


def file_stem(name):
    return re.sub(r"[^a-z0-9]+", "_", name.replace("'", "").lower()).strip("_")


def notes_relative(name):
    return f"docs/card-notes/{file_stem(name)}.md"


def card_relative(name):
    """The card's Rust file, even when it was just deleted (a card of core keywords needs none)."""
    return f"crates/cantrip-cards/src/cards/{file_stem(name)}.rs"


def commit_kind(path, content_rel, card_rel, engine, notes_rel=None):
    """Which commit a changed path belongs to, or `core` (refused without `--engine`) or `other`."""
    if path == content_rel or (card_rel and path == card_rel) or (notes_rel and path == notes_rel):
        return "card"
    if path.startswith("scenarios/") and path.endswith(".cantrip"):
        return "card"
    if path == KNOWN_ISSUES:
        return "card"
    if under(path, MECHANICS_DIRS):
        return "mechanics"
    if under(path, ORACLE_DIRS) or under(path, HARNESS_DIRS) or path in HARNESS_DOCS:
        return "engine"
    if under(path, ENGINE_DIRS):
        return "engine" if engine else "core"
    if under(path, CLIENT_DIRS) or path in CLIENT_DOCS:
        return "client"
    return "other"


def content_card(path):
    if not path or not os.path.isfile(path):
        return None
    text = "\n".join(line for line in read_text(path).splitlines() if not line.lstrip().startswith("//"))
    try:
        card = json.loads(text)
    except json.JSONDecodeError as error:
        raise GateFailed(f"{path} is not JSON: {error}")
    return card if isinstance(card, dict) else None


def printed_lines(content):
    """Every printed Oracle line, from every face the content file defines."""
    card = content_card(content)
    if not card:
        return []
    lines = []

    def add(text):
        if isinstance(text, str):
            lines.extend(line.strip() for line in text.splitlines() if line.strip())

    add(card.get("oracleText"))
    for key in ("backFace", "adventure", "omen", "aftermath"):
        face = card.get(key)
        if isinstance(face, dict):
            add(face.get("oracleText"))
    for half in card.get("halves") or []:
        if isinstance(half, dict):
            add(half.get("oracleText"))
    meld = card.get("meld")
    if isinstance(meld, dict) and isinstance(meld.get("into"), dict):
        add(meld["into"].get("oracleText"))
    return list(dict.fromkeys(lines))


def content_costs_mana(content):
    card = content_card(content)
    if not card:
        return False
    faces = [card] + [card.get(key) for key in ("backFace", "adventure", "omen", "aftermath")]
    faces += [half for half in card.get("halves") or [] if isinstance(half, dict)]
    return any(isinstance(face, dict) and str(face.get("manaCost") or "").strip() for face in faces)


def reference_points(reference):
    block = section(reference, "=== Test points")
    points = []
    for line in block.splitlines():
        match = re.match(r"\s*\[\d+\]\s+(.+)$", line)
        if match:
            points.append(match.group(1).strip())
    return points


def reference_rulings(reference):
    block = section(reference, "=== Scryfall")
    rulings = []
    for line in block.splitlines():
        match = re.match(r"\s+-\s+\d{4}-\d{2}-\d{2}\s+(.+)$", line)
        if match:
            rulings.append(match.group(1).strip())
    return rulings


def report_skeleton(reference, content, issue=0):
    return {
        "issue": int(issue or 0),
        "decision": "todo",
        "reason": "",
        "validation": [{"line": line, "code": "", "agrees": True} for line in printed_lines(content)],
        "test_points": [{"point": point, "scenario": "", "line": 0} for point in reference_points(reference)],
        "rulings": [{"ruling": ruling, "scenario": "", "line": 0, "no_decision": ""}
                    for ruling in reference_rulings(reference)],
        "look_answers": [],
        "equivalent_mutants": [],
        "notes": "",
    }


def report_file(root, store, name):
    return os.path.join(packet_dir(root, store, name), REPORT)


def check_file(root, store, name):
    return os.path.join(packet_dir(root, store, name), CHECK)


def holds_card(text, name):
    pattern = r"^\s*(?:\d+\s+)?" + re.escape(name) + r"(?:\s+x\d+)?\s*$"
    return re.search(pattern, text, re.M | re.I) is not None


def scenario_line_problem(root, scenario, number, name):
    """Why a scenario line is not an expectation that catches a test point, or None."""
    scenario = str(scenario or "").replace("\\", "/")
    path = os.path.join(root, scenario.replace("/", os.sep))
    if not str(scenario).startswith("scenarios/") or not str(scenario).endswith(".cantrip"):
        return f"{scenario} is not a scenario under scenarios/"
    if not os.path.isfile(path):
        return f"{scenario} does not exist"
    try:
        number = int(number)
    except (TypeError, ValueError):
        return f"{scenario}: line is not a number"
    lines = read_text(path).splitlines()
    if number < 1 or number > len(lines):
        return f"{scenario}:{number} is not a line of the file"
    text = lines[number - 1]
    if not text.strip() or text.lstrip().startswith("#"):
        return f"{scenario}:{number} is blank or a comment"
    if re.match(r"\s*[?!]", text):
        return None if holds_card("\n".join(lines), name) else f"{scenario} does not hold {name}"
    heading = None
    for earlier in reversed(lines[:number]):
        match = re.match(r"\s*\[([^]]+)]", earlier)
        if match:
            heading = match.group(1).strip()
            break
    if heading != "Expect":
        return f"{scenario}:{number} is not a ? check or an [Expect] line"
    if "=" not in text:
        return f"{scenario}:{number} asserts nothing"
    return None if holds_card("\n".join(lines), name) else f"{scenario} does not hold {name}"


def gate_scope(root, card, claim, reference):
    entries = changed_paths(root)
    content = content_from(reference, root)
    if content is None:
        raise GateFailed("the packet has no content file: run new-card.py and `next --refresh`")
    content_rel = clean_relative(root, content)
    data = content_card(content)
    if data is None:
        raise GateFailed(f"{content_rel} is not a card object")
    card_file = card_file_from(reference, root)
    card_rel = clean_relative(root, card_file) if card_file else card_relative(data["name"])
    notes = notes_relative(data["name"])
    if os.path.exists(os.path.join(root, notes)):
        raise GateFailed(f"{notes} is still there: follow the owner's design and delete it in the card's commit")
    kinds, core, other = {}, [], []
    for entry in entries:
        kind = commit_kind(entry["path"], content_rel, card_rel, bool((claim or {}).get("engine")), notes)
        if kind == "core":
            core.append(entry["path"])
        elif kind == "other":
            other.append(entry["path"])
        else:
            kinds[entry["path"]] = kind
    if core:
        raise GateFailed("the core is closed to a card not opened `--engine`: " + ", ".join(core[:5]),
                         "Release it and open it again with `next --engine` when the coordinator has put the core"
                         " in scope.")
    if other:
        raise GateFailed("these changed paths are neither the card's nor a shared mechanic, client or harness: "
                         + ", ".join(other[:8]))
    card_paths = [path for path, kind in kinds.items() if kind == "card"]
    scenarios = [path for path in card_paths if path.startswith("scenarios/")]
    if not card_paths:
        raise GateFailed("nothing of the card is changed: its content, Rust file or a scenario")
    if not scenarios:
        raise GateFailed("no scenario is changed: the card needs one that shows it working")
    holding = [path for path in scenarios
               if os.path.isfile(os.path.join(root, path.replace("/", os.sep)))
               and holds_card(read_text(os.path.join(root, path.replace("/", os.sep))), data["name"])]
    if not holding:
        raise GateFailed(f"no changed scenario holds {data['name']}: the card's own scenario must name it in a deck")
    counts = {kind: sum(1 for value in kinds.values() if value == kind)
              for kind in ("card", "mechanics", "engine", "client")}
    summary = ", ".join(f"{n} {kind}" for kind, n in counts.items() if n) + " changed"
    return summary, {"content": content_rel, "card": card_rel, "scenarios": scenarios}


def gate_oracle(root, card):
    tool = os.path.join(root, "tools", "maintenance", "new-card.py")
    if not os.path.isfile(tool):
        raise GateFailed(f"new-card.py is missing at {clean_relative(root, tool)}")
    result = run([sys.executable, tool, card["name"], "--no-write"], root)
    out = (result.stdout or "") + ("\n" + result.stderr if result.stderr.strip() else "")
    if "DIFFERS from Scryfall" in out or "no face of that name on Scryfall" in out:
        raise GateFailed("the content's Oracle text is not Scryfall's current text", out)
    if "not found on Scryfall (or Scryfall unreachable)" in out:
        return "Scryfall unreachable: the Oracle text was checked by hand", out
    if result.returncode != 0 or "Oracle text matches Scryfall's" not in out:
        raise GateFailed("new-card.py did not confirm the Oracle text", out)
    return "Oracle text matches Scryfall's", out


def gate_scenarios(root, card, content, reference):
    paths = [entry["path"] for entry in changed_paths(root)
             if entry["path"].startswith("scenarios/") and entry["path"].endswith(".cantrip")]
    problems, holding, mana_checked = [], 0, False
    for relative in paths:
        path = os.path.join(root, relative.replace("/", os.sep))
        if not os.path.isfile(path):
            continue
        text = read_text(path)
        if not text.lstrip("\ufeff \t\r\n").startswith("#"):
            problems.append(f"{relative}: starts with the comment saying what it shows and the rule")
        if re.search(r"^\s*#\s*TODO\b", text, re.M):
            problems.append(f"{relative}: has a TODO comment")
        if re.search(r"^\s*#\s*Known bug\b", text, re.M | re.I):
            problems.append(f"{relative}: a known bug is not landed in this lane; fix it or block the card")
        if "[Expect]" not in text:
            problems.append(f"{relative}: has no [Expect] section")
        else:
            expect = text.split("[Expect]", 1)[1]
            if re.search(r"\bTODO\b", expect):
                problems.append(f"{relative}: [Expect] still has a TODO")
            if not any(line.strip() and not line.lstrip().startswith("#") for line in expect.splitlines()):
                problems.append(f"{relative}: [Expect] asserts nothing")
        if holds_card(text, card["name"]):
            holding += 1
        if ".Mana =" in text:
            mana_checked = True
    if not paths:
        problems.append("no scenario is changed")
    if paths and not holding:
        problems.append(f"no changed scenario holds {card['name']}")
    cost_point = any("the cost '" in point for point in reference_points(reference))
    if paths and (content_costs_mana(content) or cost_point) and not mana_checked:
        problems.append("no changed scenario checks the cost paid exactly: `? Bot1.Mana =` right after the cast")
    if problems:
        raise GateFailed(f"{len(problems)} scenario problem(s)", "\n".join(problems))
    return f"{len(paths)} scenario(s), {holding} holding {card['name']}", ""


def gate_corpus(root, card, changed):
    command = ["cargo", "run", "--profile", "cards", "-p", "cantrip-script", "--bin", "corpus",
               "--", "--card", card["name"], "--brief"]
    print(f"corpus: running {card['name']}'s scenarios ...", file=sys.stderr, flush=True)
    result = run(command, root, env=dict(os.environ, CANTRIP_VERIFY="1"))
    out = (result.stdout or "") + ("\n" + result.stderr if result.stderr.strip() else "")
    found = re.search(r"(\d+) scenarios: (\d+) pass, (\d+) fail, (\d+) known bugs, (\d+) need cards", out)
    if not found:
        raise GateFailed("the corpus did not report a summary", out[-8000:])
    total, passed, failed, known, blocked = (int(value) for value in found.groups())
    warnings = []
    for block in re.split(r"(?=^warning:)", out, flags=re.M)[1:]:
        if any(path in block for path in changed):
            warnings.append(block.strip()[:1200])
    if warnings:
        raise GateFailed(f"the build warns from the card's files ({len(warnings)})", "\n\n".join(warnings))
    if result.returncode != 0 or failed or known or blocked:
        raise GateFailed(f"{passed} pass, {failed} fail, {known} known bug(s), {blocked} need cards", out[-8000:])
    if passed == 0:
        raise GateFailed("no scenario passed: the card has no scenario that holds it", out[-4000:])
    return f"{passed} pass, 0 fail ({total} scenario(s))", out


def gate_lint(root, card):
    lint = os.path.join(root, "tools", "maintenance", "card-lint.py")
    dupes = os.path.join(root, "tools", "maintenance", "card-dupes.py")
    if not os.path.isfile(lint) or not os.path.isfile(dupes):
        raise GateFailed("card-lint.py or card-dupes.py is missing")
    linted = run([sys.executable, lint, card["name"]], root)
    doubled = run([sys.executable, dupes, card["name"]], root)
    out = (linted.stdout or "") + (linted.stderr or "") + (doubled.stdout or "") + (doubled.stderr or "")
    looks, never = [], []
    for line in out.splitlines():
        match = re.search(r"([^\s:]+\.rs):(\d+): (NEVER|LOOK): (.*)", line)
        if match:
            (never if match.group(3) == "NEVER" else looks).append(
                {"where": f"{match.group(1)}:{match.group(2)}", "text": match.group(4).strip()})
        shared = re.search(r"fn (\w+): the same body as", line)
        if shared:
            looks.append({"where": f"fn {shared.group(1)}", "text": line.strip()})
    if linted.returncode != 0 or never:
        raise GateFailed(f"card-lint has {len(never)} NEVER finding(s)", out)
    if doubled.returncode != 0:
        raise GateFailed("card-dupes.py failed", out)
    summary = "clean" if not looks else f"{len(looks)} LOOK finding(s), each answered in {REPORT}"
    return summary, {"looks": looks, "output": out}


def gate_citations(root, card, reference):
    tool = os.path.join(root, "tools", "maintenance", "cr.py")
    if not os.path.isfile(tool):
        raise GateFailed("cr.py is missing")
    paths = []
    card_file = card_file_from(reference, root)
    if card_file:
        paths.append(clean_relative(root, card_file))
    scenario_dir = os.path.join(root, "scenarios")
    if os.path.isdir(scenario_dir):
        for entry in sorted(os.listdir(scenario_dir)):
            if entry.endswith(".cantrip"):
                path = os.path.join(scenario_dir, entry)
                if holds_card(read_text(path), card["name"]):
                    paths.append(clean_relative(root, path))
    if not paths:
        return "no card file or scenario to cite", ""
    result = run([sys.executable, tool, "check", "--show", *paths], root)
    out = (result.stdout or "") + ("\n" + result.stderr if result.stderr.strip() else "")
    if result.returncode != 0:
        raise GateFailed("a cited rule does not exist", out)
    return "every cited rule exists", out


def gate_coverage(root, card):
    tool = os.path.join(root, "tools", "maintenance", "card-coverage.py")
    if not os.path.isfile(tool):
        raise GateFailed("card-coverage.py is missing")
    print(f"coverage: running card-coverage.py for {card['name']} ...", file=sys.stderr, flush=True)
    result = run([sys.executable, tool, card["name"]], root)
    out = (result.stdout or "") + ("\n" + result.stderr if result.stderr.strip() else "")
    if result.returncode != 0:
        raise GateFailed("some lines of the card never ran", out)
    lines = [line for line in out.splitlines() if line.strip()]
    return (lines[-1].strip() if lines else "every line ran"), out


def equivalent_excuse(report, survivor):
    where = f"{survivor['path']}:{survivor['line']}"
    for excuse in (report or {}).get("equivalent_mutants") or []:
        if not isinstance(excuse, dict):
            continue
        mutant = str(excuse.get("mutant") or "")
        why = str(excuse.get("why") or "").strip()
        if where in mutant and len(why) >= 20:
            return why
    return None


def gate_mutation(root, card, report):
    tool = os.path.join(root, "tools", "maintenance", "card-mutate.py")
    if not os.path.isfile(tool):
        raise GateFailed("card-mutate.py is missing")
    print(f"mutation: running card-mutate.py for {card['name']} ...", file=sys.stderr, flush=True)
    result = run([sys.executable, tool, card["name"]], root)
    out = (result.stdout or "") + ("\n" + result.stderr if result.stderr.strip() else "")
    summary = re.search(r"(\d+) caught, (\d+) survived, (\d+) did not build", out)
    survivors = [{"path": match.group(1), "line": match.group(2), "what": match.group(3).strip()}
                 for match in re.finditer(r"^SURVIVED (\S+):(\d+): (.+)$", out, re.M)]
    if result.returncode == 0:
        if summary and int(summary.group(2)):
            raise GateFailed("card-mutate.py exited 0 with survivors", out)
        caught = int(summary.group(1)) if summary else 0
        return f"{caught} caught, 0 survived", {"detail": out, "survivors": []}
    if not summary:
        raise GateFailed("card-mutate.py failed before reporting", out)
    caught, survived, unbuilt = (int(value) for value in summary.groups())
    if unbuilt:
        raise GateFailed(f"{unbuilt} mutation(s) did not build", out)
    if len(survivors) != survived:
        raise GateFailed(f"card-mutate.py reported {survived} survivor(s) but {len(survivors)} could be read", out)
    if not survived:
        raise GateFailed("card-mutate.py exited 1 but reported no survivor", out)
    missing = [survivor for survivor in survivors if not equivalent_excuse(report, survivor)]
    if missing:
        detail = "\n".join(f"{s['path']}:{s['line']}: {s['what']}" for s in missing)
        raise GateFailed(f"{len(missing)} mutation survivor(s) no scenario catches and the report does not excuse",
                         detail + "\n\n" + out[-5000:])
    return f"{caught} caught, {survived} survived (all declared equivalent)", {"detail": out, "survivors": survivors}


def gate_report(root, card, content, reference, report, looks, survivors, issue=0):
    if not isinstance(report, dict):
        raise GateFailed(f"{REPORT} is missing or not an object: fill it in from the packet")
    problems = []
    if report.get("decision") != "done":
        problems.append(f"decision is {report.get('decision')!r}: finish the card and set it to \"done\"")
    report_issue = report.get("issue")
    try:
        report_issue = int(report_issue)
    except (TypeError, ValueError):
        report_issue = 0
    if report_issue <= 0 and not issue:
        problems.append("no issue number: set report.json's \"issue\" or pass --issue N")
    seen = {}
    for entry in report.get("validation") or []:
        if isinstance(entry, dict) and isinstance(entry.get("line"), str):
            seen[entry["line"].strip()] = entry
    for line in printed_lines(content):
        entry = seen.get(line.strip())
        if entry is None:
            problems.append(f"no validation entry for the printed line: {line}")
        elif entry.get("agrees") is not True:
            problems.append(f"the report says the code disagrees with: {line}")
        elif len(str(entry.get("code") or "").strip()) < 8:
            problems.append(f"the validation line has no code that implements it: {line}")
    points = [entry for entry in report.get("test_points") or [] if isinstance(entry, dict)]
    for point in reference_points(reference):
        entry = next((entry for entry in points if str(entry.get("point")).strip() == point), None)
        if entry is None:
            problems.append(f"no test-point entry for: {point}")
            continue
        problem = scenario_line_problem(root, entry.get("scenario"), entry.get("line"), card["name"])
        if problem:
            problems.append(f"test point {point!r}: {problem}")
    rulings = [entry for entry in report.get("rulings") or [] if isinstance(entry, dict)]
    for ruling in reference_rulings(reference):
        entry = next((entry for entry in rulings if str(entry.get("ruling")).strip() == ruling), None)
        if entry is None:
            problems.append(f"no ruling entry for: {ruling[:100]}")
            continue
        if str(entry.get("scenario") or "").strip():
            problem = scenario_line_problem(root, entry.get("scenario"), entry.get("line"), card["name"])
            if problem:
                problems.append(f"ruling {ruling[:60]!r}: {problem}")
        elif len(str(entry.get("no_decision") or "").strip()) < 20:
            problems.append(f"ruling {ruling[:60]!r}: give the scenario line that decides it, or say in "
                            "no_decision why the card does not decide it")
    answers = [answer for answer in report.get("look_answers") or [] if isinstance(answer, str)]
    for look in looks:
        if not any(look["where"] in answer and len(answer.strip()) >= 20 for answer in answers):
            problems.append(f"no report answer for {look['where']}: {look['text'][:120]}")
    for survivor in survivors:
        if not equivalent_excuse(report, survivor):
            problems.append(f"mutation survivor {survivor['path']}:{survivor['line']} is not declared equivalent")
    if problems:
        raise GateFailed(f"{len(problems)} problem(s) in {REPORT}", "\n".join(problems))
    return (f"{len(printed_lines(content))} printed line(s), {len(reference_points(reference))} test point(s), "
            f"{len(reference_rulings(reference))} ruling(s)"), ""


def check_card(root, store, card, claim, reference, fast=False):
    """Every gate, in order; fast skips only the two that rebuild and mutate the card."""
    gates = []
    changed = [entry["path"] for entry in changed_paths(root)]

    def add(name, function):
        try:
            summary, facts = function()
            status = "pass"
        except GateFailed as error:
            status, summary, facts = "fail", str(error), {"detail": error.detail}
        except Exception as error:  # a broken tool is a failed gate, never a pass
            status, summary, facts = "fail", f"{type(error).__name__}: {error}", {"detail": ""}
        if status == "pass" and summary.startswith("Scryfall unreachable"):
            status = "warn"
        if isinstance(facts, dict):
            detail = str(facts.get("detail") or "")
            kept = facts
        elif isinstance(facts, str):
            detail, kept = facts, {}
        else:
            detail, kept = "", {}
        gates.append({"name": name, "status": status, "summary": summary,
                      "detail": detail[:20000], "facts": kept})
        return gates[-1]

    add("scope", lambda: gate_scope(root, card, claim, reference))
    content = content_from(reference, root)
    add("oracle", lambda: gate_oracle(root, card))
    add("scenarios", lambda: gate_scenarios(root, card, content, reference))
    add("corpus", lambda: gate_corpus(root, card, changed))
    linted = add("lint", lambda: gate_lint(root, card))
    add("citations", lambda: gate_citations(root, card, reference))
    if fast:
        gates.append({"name": "coverage", "status": "skip", "summary": "--fast", "detail": "", "facts": {}})
    else:
        add("coverage", lambda: gate_coverage(root, card))
    if fast:
        gates.append({"name": "mutation", "status": "skip", "summary": "--fast", "detail": "", "facts": {}})
        mutation = {"facts": {}}
    else:
        report_data = {}
        if os.path.isfile(report_file(root, store, card["name"])):
            try:
                report_data = read_json(report_file(root, store, card["name"]))
            except json.JSONDecodeError:
                report_data = {}
        mutation = add("mutation", lambda: gate_mutation(root, card, report_data))
    report = None
    try:
        report = read_json(report_file(root, store, card["name"]))
    except FileNotFoundError:
        gates.append({"name": "report", "status": "fail", "summary": f"{REPORT} is missing: fill it in",
                      "detail": "", "facts": {}})
    except json.JSONDecodeError as error:
        gates.append({"name": "report", "status": "fail", "summary": f"{REPORT} is not JSON: {error}",
                      "detail": "", "facts": {}})
    if report is not None:
        looks = (linted.get("facts") or {}).get("looks") or []
        survivors = (mutation.get("facts") or {}).get("survivors") or []
        add("report", lambda: gate_report(root, card, content, reference, report, looks, survivors,
                                          (claim or {}).get("issue")))
    return gates


def save_check(root, store, card, gates, fast):
    passed = all(gate["status"] != "fail" for gate in gates)
    data = {"queue": store.name, "card": card["name"], "head": head_sha(root), "tree": tree_sha(root),
            "fast": bool(fast), "time": now(), "passed": passed, "gates": gates}
    write_json(check_file(root, store, card["name"]), data)
    return passed


def print_gates(card, root, gates, fast):
    print(f"check {card['name']} in {root}" + (" (fast: coverage and mutation skipped)" if fast else ""))
    for gate in gates:
        print(f"  {gate['name']:<10} {gate['status'].upper():<5} {gate['summary']}")
    for gate in gates:
        if gate["status"] in ("fail", "warn") and gate.get("detail"):
            print(f"  {gate['name']} detail:")
            for line in gate["detail"].splitlines()[:40]:
                print(f"    {line}")
    print("check: PASS" if all(gate["status"] != "fail" for gate in gates) else "check: FAIL - nothing is committed")


def plan_commits(root, store, card, claim, reference, issue):
    """Stage by category and commit Engine, Mechanics, Client, then exactly one Card commit."""
    entries = changed_paths(root)
    content = content_from(reference, root)
    content_rel = clean_relative(root, content) if content else None
    card_file = card_file_from(reference, root)
    card_rel = clean_relative(root, card_file) if card_file else card_relative(card["name"])
    notes = notes_relative(card["name"])
    groups = {"engine": [], "mechanics": [], "client": [], "card": []}
    for entry in entries:
        kind = commit_kind(entry["path"], content_rel, card_rel, bool(claim.get("engine")), notes)
        if kind == "core":
            raise Refused(f"{entry['path']} is core and this card was not opened --engine")
        if kind == "other":
            raise Refused(f"{entry['path']} is not the card's, a shared mechanic, the client or the harness")
        groups[kind].append(entry["path"])
    if not groups["card"]:
        raise Refused("nothing of the card is changed: its content, Rust file or a scenario")
    messages = {"engine": f"Engine: what {card['name']} needs of the engine",
                "mechanics": f"Mechanics: what {card['name']} needs of the shared mechanics",
                "client": f"Client: what {card['name']} adds to the client",
                "card": f"Card: {card['name']}\n\nCloses #{issue}"}
    made = []
    for kind in ("engine", "mechanics", "client", "card"):
        paths = groups[kind]
        if not paths:
            continue
        before = head_sha(root)
        added = git(root, "add", "-A", "--", *paths, check=False)
        if added.returncode != 0:
            raise Failed(f"git add failed:\n{(added.stdout + added.stderr).strip()}")
        message = messages[kind]
        committed = git(root, "commit", "-m", message, check=False)
        if committed.returncode != 0:
            raise Refused(f"git commit failed for {kind}:\n{(committed.stdout + committed.stderr).strip()[-3000:]}")
        after = head_sha(root)
        count = int(git(root, "rev-list", "--count", f"{before}..{after}").stdout.strip() or 0)
        body = git(root, "log", "-1", "--format=%B").stdout.rstrip()
        if count != 1:
            raise Refused(f"the {kind} commit made {count} commits, not one; inspect the branch")
        if "co-authored-by" in body.lower():
            raise Refused(f"the {kind} commit carries Co-Authored-By, which this repository forbids; amend it")
        if kind == "card":
            if not body.startswith(f"Card: {card['name']}"):
                raise Refused(f"the card commit's subject is not `Card: {card['name']}`: {body.splitlines()[0]}")
            if not re.search(rf"^Closes #{issue}$", body, re.M):
                raise Refused(f"the card commit does not end with `Closes #{issue}`")
        made.append({"kind": kind, "sha": after[:9], "message": body.splitlines()[0], "paths": paths})
    dirty = status_paths(root)
    if dirty:
        raise Refused("commits were made but the tree is still dirty:\n  " + "\n  ".join(dirty[:12]))
    return made


def cmd_check(args):
    root = worktree_root()
    store = Store(common_dir(root), args.queue)
    state = store.read()
    card, claim = None, {}
    if args.id:
        wid = worker_id(args.id)
        card = open_claim(state, wid)
        if card:
            claim = card["claim"]
            if norm(claim.get("worktree", "")) != norm(root):
                raise Refused(f"{card['name']} is open for {wid} in {claim.get('worktree')}; check it there")
            if args.card and args.card.casefold() != card["name"].casefold():
                raise Refused(f"{wid} holds {card['name']}, not {args.card}")
        elif args.card:
            card = card_of(state, args.card) or {"name": args.card}
        else:
            raise Refused(f"{wid} has no open card in queue {store.name}")
    elif args.card:
        card = card_of(state, args.card) or {"name": args.card}
    else:
        card = open_in(state, root)
        if card:
            claim = card["claim"]
        else:
            raise Refused("name the card: --id WORKER (its open card) or --card NAME")
    packet = packet_dir(root, store, card["name"])
    reference_path = os.path.join(packet, "reference.md")
    if not os.path.isfile(reference_path):
        raise Refused(f"no packet for {card['name']} at {packet}: run `next --id ...` first")
    reference = read_text(reference_path)
    gates = check_card(root, store, card, claim, reference, bool(args.fast))
    passed = save_check(root, store, card, gates, bool(args.fast))
    print_gates(card, root, gates, bool(args.fast))
    return 0 if passed else 1


def cmd_commit(args):
    wid = worker_id(args.id)
    root = worktree_root()
    store = Store(common_dir(root), args.queue)
    pool_context(root)
    require_branch(root)
    state = store.read()
    card = open_claim(state, wid)
    if card is None:
        raise Refused(f"{wid} has no open card in queue {store.name}")
    claim = card["claim"]
    if norm(claim.get("worktree", "")) != norm(root):
        raise Refused(f"{card['name']} is open for {wid} in {claim.get('worktree')}; commit it there")
    packet = packet_dir(root, store, card["name"])
    reference_path = os.path.join(packet, "reference.md")
    if not os.path.isfile(reference_path):
        raise Refused(f"no packet for {card['name']} at {packet}: run `next --id ...` first")
    reference = read_text(reference_path)
    report = {}
    try:
        report = read_json(report_file(root, store, card["name"]))
    except (FileNotFoundError, json.JSONDecodeError):
        report = {}
    issue = args.issue or claim.get("issue") or (report.get("issue") if isinstance(report, dict) else 0)
    try:
        issue = int(issue)
    except (TypeError, ValueError):
        issue = 0
    if issue <= 0:
        raise Refused("no issue number: pass --issue N or set report.json's issue (the Card commit ends Closes #N)")
    gates = check_card(root, store, card, claim, reference, fast=False)
    passed = save_check(root, store, card, gates, False)
    print_gates(card, root, gates, False)
    if not passed:
        raise Refused("the check failed; nothing was committed")
    made = plan_commits(root, store, card, claim, reference, issue)
    print(f"{card['name']}: committed")
    for commit in made:
        print(f"  {commit['sha']} {commit['message']}  ({len(commit['paths'])} path(s))")
    close_args = argparse.Namespace(id=wid, queue=store.name, commit=made[-1]["sha"], no_commit=False,
                                    note=args.note or "")
    return close(close_args, "done")


# ---- commands -------------------------------------------------------------------------------------------------------

def cmd_queue(args):
    root = worktree_root()
    store = Store(common_dir(root), args.queue)
    names = requested_names(args, root)
    implemented = implemented_names(root)
    with store.locked() as state:
        if not state["cards"]:
            state["created"] = now()
        ensure_branch(root, store)
        added, present, skipped = [], [], []
        for name in names:
            card = card_of(state, name)
            if card:
                card["listed"] = True
                present.append(name)
                continue
            if slug(name) in implemented:
                skipped.append(name)
                continue
            state["cards"].append({"name": name, "status": "pending", "listed": True, "attempts": 0,
                                   "claim": None, "commit": None, "finished": None})
            added.append(name)
        store.log(state, "queue", added=len(added), present=len(present), skipped_implemented=len(skipped))
        total = len(state["cards"])
        counts = {kind: sum(1 for card in state["cards"] if card["status"] == kind)
                  for kind in ("pending", "working", "done", "blocked", "abandoned")}
    print(f"queue {store.name} ({store.path})")
    print(f"  {total} card(s): " + ", ".join(f"{n} {kind}" for kind, n in counts.items()))
    print(f"  added {len(added)}, already queued {len(present)}, already implemented {len(skipped)}")
    for label, values in (("added", added), ("implemented, skipped", skipped)):
        for name in values[:12]:
            print(f"  {label}: {name}")
        if len(values) > 12:
            print(f"  {label}: ... and {len(values) - 12} more")
    print(f"  shared branch: {store.branch} at {(branch_tip(root, store.branch) or '?')[:9]}")
    return 0


def cmd_next(args):
    wid = worker_id(args.id)
    issue = None
    if getattr(args, "issue", None) is not None:
        try:
            issue = int(args.issue)
        except (TypeError, ValueError):
            raise Refused("--issue takes a GitHub issue number, as --issue 123")
        if issue <= 0:
            raise Refused("--issue takes a positive GitHub issue number")
    root = worktree_root()
    store = Store(common_dir(root), args.queue)
    context = pool_context(root)
    require_branch(root)
    fresh, opened, card, claim, packet, brief = False, False, None, None, None, None
    with store.locked() as state:
        card = open_claim(state, wid)
        if card:
            claim = card["claim"]
            if issue and not claim.get("issue"):
                claim["issue"] = issue
            moved = norm(claim.get("worktree", "")) != norm(root)
            if moved:
                if os.path.isdir(claim.get("worktree", "")):
                    raise Refused(f"{wid} already has {card['name']} open in {claim['worktree']}. A card is worked in "
                                  f"one worktree: finish it there, or `release --id {wid} --reason \"...\"` first.")
                claim.update(worktree=root, packet=packet_dir(root, store, card["name"]),
                             slot=context["slot"], lease=context["lease"], member=context["member"],
                             agent=context["agent"])
                store.log(state, "adopt", id=wid, card=card["name"], worktree=root)
            packet = packet_dir(root, store, card["name"])
            fresh = moved or args.refresh or not os.path.isfile(os.path.join(packet, "brief.md"))
        else:
            busy = open_in(state, root)
            if busy:
                raise Refused(f"this worktree already holds {busy['name']} open for {busy['claim']['id']}. One open "
                              f"card per worktree: `done`, `block` or `release` it first.")
            require_clean(root, "before opening a card")
            if args.card:
                card = card_of(state, args.card)
                if card is None:
                    raise Refused(f"{args.card} is not in queue {store.name}; add it with `queue --card \"...\"`")
                if card["status"] == "done":
                    raise Refused(f"{card['name']} is already done in queue {store.name}")
                if card["status"] == "working" and (card.get("claim") or {}).get("id") != wid:
                    raise Refused(f"{card['name']} is being worked by {card['claim']['id']}")
            else:
                card = next((entry for entry in state["cards"] if entry["status"] == "pending"), None)
                if card is None:
                    counts = {kind: sum(1 for entry in state["cards"] if entry["status"] == kind)
                              for kind in ("working", "done", "blocked", "abandoned")}
                    raise Refused(f"queue {store.name} has no pending card (" +
                                  ", ".join(f"{n} {kind}" for kind, n in counts.items()) + ")")
            card.update(status="working", attempts=card.get("attempts", 0) + 1, started=now(),
                        commit=None, finished=None, failure=None)
            card["claim"] = {"id": wid, "worktree": root, "packet": packet_dir(root, store, card["name"]),
                             "slot": context["slot"], "lease": context["lease"], "member": context["member"],
                             "agent": context["agent"], "head": head_sha(root), "engine": bool(getattr(args, "engine", False)),
                             "issue": issue}
            store.log(state, "claim", id=wid, card=card["name"], worktree=root, slot=context["slot"],
                      lease=context["lease"], member=context["member"], engine=bool(getattr(args, "engine", False)))
            claim, fresh, opened = card["claim"], True, True
            packet = packet_dir(root, store, card["name"])
    try:
        if fresh:
            if context["slot"] is None:
                print(f"note: {root} is not a pool slot; this is a solo claim.", file=sys.stderr)
            if opened:
                require_clean(root, "before opening a card")
            merged = merge_branch(root, store)
            claim["head"] = head_sha(root)
            with store.locked() as state:
                recorded = open_claim(state, wid)
                if recorded:
                    recorded["claim"]["head"] = claim["head"]
                    if merged:
                        store.log(state, "sync", id=wid, card=recorded["name"], branch=store.branch, tip=merged)
            packet, brief = build_packet(root, store, card, claim)
        else:
            with open(os.path.join(packet, "brief.md"), encoding="utf-8") as f:
                brief = f.read()
    except Exception as error:
        with store.locked() as state:
            recorded = open_claim(state, wid)
            if recorded and recorded["name"] == card["name"]:
                recorded.update(status="pending", claim=None, failure=f"{type(error).__name__}: {error}"[:600])
                store.log(state, "packet-failed", id=wid, card=card["name"], error=str(error)[:300])
        raise
    print(brief, end="")
    return 0


def cmd_status(args):
    root = worktree_root()
    common = common_dir(root)
    store = Store(common, args.queue)
    state = store.read()
    leases, members = pool_claims(common)
    tip = branch_tip(root, store.branch)
    counts = {kind: sum(1 for card in state["cards"] if card["status"] == kind)
              for kind in ("working", "done", "blocked", "abandoned", "pending")}
    print(f"queue {store.name} ({store.path})")
    print(f"  {len(state['cards'])} card(s): " + ", ".join(f"{n} {kind}" for kind, n in counts.items() if n or kind != "pending"))
    if tip:
        ahead = 0
        if branch_tip(root, "origin/main"):
            ahead = int(git(root, "rev-list", "--count", f"origin/main..{tip}", check=False).stdout.strip() or 0)
        print(f"  shared branch {store.branch} at {tip[:9]}: {ahead} commit(s) not on origin/main")
    else:
        print(f"  shared branch {store.branch}: not created yet (the first claim makes it)")
    working = [card for card in state["cards"] if card["status"] == "working"]
    if args.id:
        working = [card for card in working if card["claim"]["id"] == args.id]
    if working:
        print("working:")
    for card in working:
        claim = card["claim"]
        where = claim.get("slot") or claim.get("worktree")
        stale = []
        if claim.get("lease") and claim["lease"] not in leases:
            stale.append("lease released")
        if claim.get("member") and claim["member"] not in members:
            stale.append("member gone")
        if not os.path.isdir(claim.get("worktree", "")):
            stale.append("worktree gone")
        packet = claim.get("packet", "")
        print(f"  {card['name']}  [{claim['id']}]  {where}  since {card.get('started', '?')}"
              + (f"  ({', '.join(stale)})" if stale else "")
              + (f"\n    packet: {packet}" if packet and os.path.isdir(packet) else "\n    packet: missing"))
    blocked = [card for card in state["cards"] if card["status"] == "blocked"]
    for card in blocked:
        print(f"blocked: {card['name']}" + (f" (CR {card.get('rule')})" if card.get("rule") else ""))
    pending = [card["name"] for card in state["cards"] if card["status"] == "pending"]
    if pending:
        print("pending: " + ", ".join(pending[:10]) + (f" (+{len(pending) - 10} more)" if len(pending) > 10 else ""))
    return 0


def close(args, outcome):
    """`done`, `block`, `abandon` and `release` in one shape: validate, share, then close the claim."""
    wid = worker_id(args.id)
    root = worktree_root()
    store = Store(common_dir(root), args.queue)
    pool_context(root)
    requires_commit = outcome == "done"
    with store.locked() as state:
        card = open_claim(state, wid)
        if card is None:
            raise Refused(f"{wid} has no open card in queue {store.name}")
        claim = card["claim"]
        if norm(claim.get("worktree", "")) != norm(root):
            raise Refused(f"{card['name']} is open for {wid} in {claim.get('worktree')}; close it there")
        if requires_commit:
            require_clean(root, "before the card is marked done")
            head = head_sha(root)
            commit = args.commit or head
            if git(root, "cat-file", "-e", f"{commit}^{{commit}}", check=False).returncode != 0:
                raise Refused(f"{commit} is not a commit in {root}")
            if not ancestor(root, commit, "HEAD"):
                raise Refused(f"{commit} is not on this worktree's branch")
            if commit == claim.get("head") and not args.no_commit:
                raise Refused("no commit has been made since the card was opened; commit the card, or pass "
                              "--no-commit when it needed none (already implemented, for example)")
        else:
            commit = None
    packet, untracked = save_patch(root, store, card["name"])
    shared = []
    try:
        new, shared = publish(root, store)
    except Refused as error:
        if requires_commit:
            raise
        print(f"warning: the claim is closing but {store.branch} was not updated: {error}", file=sys.stderr)
        new = branch_tip(root, store.branch)
    with store.locked() as state:
        card = open_claim(state, wid)
        if card is None:
            raise Refused(f"{wid} lost its claim while {outcome} was being recorded")
        card["status"] = "done" if outcome == "done" else ("blocked" if outcome == "block" else
                                                           ("abandoned" if outcome == "abandon" else "pending"))
        card["commit"] = commit
        card["finished"] = now()
        card["failure"] = None
        if outcome == "block":
            card["rule"] = args.rule
        if outcome in ("block", "abandon", "release"):
            card["note"] = args.note if outcome != "release" else args.reason
        card["claim"] = None
        store.log(state, outcome, id=wid, card=card["name"], commit=commit, shared=new,
                  note=(getattr(args, "note", None) or getattr(args, "reason", None) or ""))
    what = {"done": f"done, on {store.branch} at {(new or '?')[:9]}", "block": "blocked",
            "abandon": "abandoned", "release": "back in the queue (pending)"}[outcome]
    print(f"{card['name']}: {what}.")
    if shared:
        print("  shared now on " + store.branch + ":")
        for line in shared[:10]:
            print(f"    {line}")
    if packet:
        print(f"  uncommitted work saved to {os.path.join(packet, 'work.patch')}")
    if untracked:
        print("  untracked files were left in the worktree:")
        for path in untracked[:10]:
            print(f"    {path}")
    if outcome == "done":
        print(f"  Packet: {packet_dir(root, store, card['name'])}")
    return 0


def cmd_done(args):
    print("note: `done` closes without the check and commit gates; `commit` is the gated path.", file=sys.stderr)
    return close(args, "done")


def cmd_block(args):
    return close(args, "block")


def cmd_abandon(args):
    return close(args, "abandon")


def cmd_release(args):
    return close(args, "release")


def cmd_share(args):
    wid = worker_id(args.id)
    root = worktree_root()
    store = Store(common_dir(root), args.queue)
    pool_context(root)
    with store.locked() as state:
        card = open_claim(state, wid)
        if card is None:
            raise Refused(f"{wid} has no open card in queue {store.name}")
        if norm(card["claim"].get("worktree", "")) != norm(root):
            raise Refused(f"{card['name']} is open for {wid} in {card['claim'].get('worktree')}")
    new, shared = publish(root, store)
    with store.locked() as state:
        recorded = open_claim(state, wid)
        if recorded and recorded["name"] == card["name"]:
            recorded["claim"]["shared"] = new
        store.log(state, "share", id=wid, card=card["name"], branch=store.branch, tip=new)
    print(f"shared {store.branch} at {new[:9]}: " + (f"{len(shared)} new commit(s)" if shared else "nothing new"))
    for line in shared[:10]:
        print(f"  {line}")
    print("the next implementor's `next` merges it into their worktree.")
    return 0


# ---- the command line -----------------------------------------------------------------------------------------------

def command_line(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)

    def queue_flag(p):
        p.add_argument("--queue", default=DEFAULT_QUEUE, metavar="NAME",
                       help=f"the shared queue (default {DEFAULT_QUEUE}); its branch is card-lane/<NAME>")

    cmd = sub.add_parser("queue", help="make or extend the shared queue")
    queue_flag(cmd)
    cmd.add_argument("--format", metavar="F", help="docs/formats/<F>.json, in its own order")
    cmd.add_argument("--file", metavar="PATH", help="a card list, one name a line")
    cmd.add_argument("--card", action="append", default=[], metavar="NAME", help="one card (repeatable)")
    cmd.add_argument("names", nargs="*", metavar="NAME", help="card names")
    cmd.set_defaults(handler=cmd_queue)

    cmd = sub.add_parser("next", help="claim the next card, or print this id's open card again")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.add_argument("--card", metavar="NAME", help="take this card specifically")
    cmd.add_argument("--refresh", action="store_true", help="rebuild the packet even if it is there")
    cmd.add_argument("--engine", action="store_true", help="this card may change the core (state, rules, api)")
    cmd.add_argument("--issue", type=int, metavar="N", help="the GitHub issue the Card commit closes")
    cmd.set_defaults(handler=cmd_next)

    cmd = sub.add_parser("status", help="the queue, its claims and the shared branch")
    queue_flag(cmd)
    cmd.add_argument("--id", metavar="WORKER")
    cmd.set_defaults(handler=cmd_status)

    cmd = sub.add_parser("check", help="run the card's gates (fast skips coverage and mutation)")
    queue_flag(cmd)
    cmd.add_argument("--id", metavar="WORKER", help="check this id's open card")
    cmd.add_argument("--card", metavar="NAME", help="check this card's packet")
    cmd.add_argument("--fast", action="store_true", help="skip only coverage and mutation")
    cmd.set_defaults(handler=cmd_check)

    cmd = sub.add_parser("commit", help="gate the card, commit it in the add-cards shape, and close it")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.add_argument("--issue", type=int, metavar="N", help="the GitHub issue the Card commit closes")
    cmd.add_argument("--note", metavar="TEXT")
    cmd.set_defaults(handler=cmd_commit)

    cmd = sub.add_parser("done", help="close the claim without the gates (commit is the gated path)")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.add_argument("--commit", metavar="SHA", help="the card's commit (default HEAD)")
    cmd.add_argument("--no-commit", action="store_true", help="the card needed no commit (already implemented)")
    cmd.add_argument("--note", metavar="TEXT")
    cmd.set_defaults(handler=cmd_done)

    cmd = sub.add_parser("block", help="the card waits on an engine gap: close the claim with its rule")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.add_argument("--rule", required=True, metavar="CR", help="the Comprehensive Rules number it waits on")
    cmd.add_argument("--note", metavar="TEXT")
    cmd.set_defaults(handler=cmd_block)

    cmd = sub.add_parser("release", help="put the card back in the queue for someone else")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.add_argument("--reason", required=True, metavar="TEXT")
    cmd.set_defaults(handler=cmd_release)

    cmd = sub.add_parser("abandon", help="the card will not be built: close it for good")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.add_argument("--reason", required=True, metavar="TEXT")
    cmd.set_defaults(handler=cmd_abandon)

    cmd = sub.add_parser("share", help="put this worktree's commits on the queue's branch now")
    queue_flag(cmd)
    cmd.add_argument("--id", required=True, metavar="WORKER")
    cmd.set_defaults(handler=cmd_share)
    return parser


def main(argv=None):
    args = command_line(argv).parse_args(argv)
    try:
        return args.handler(args)
    except Refused as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 2
    except Failed as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
