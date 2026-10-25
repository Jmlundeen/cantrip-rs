"""The worktree pool: the warm worktrees agents work in, and who may work in which.

    python tools/maintenance/worktrees.py create [--count N] [--no-warm]
    python tools/maintenance/worktrees.py acquire --for "<purpose>" [--name X]
    python tools/maintenance/worktrees.py token <lease-id> [--read-only] [--name X]
    python tools/maintenance/worktrees.py join <token> --agent-id <id>
    python tools/maintenance/worktrees.py leave <member-id>
    python tools/maintenance/worktrees.py sync <lease-or-member-id>
    python tools/maintenance/worktrees.py evict <lease-id> --member <member-id> --reason "..."
    python tools/maintenance/worktrees.py release <lease-id> [--discard --reason "..."]
    python tools/maintenance/worktrees.py clean <slot> --reason "..." [--discard-commits]
    python tools/maintenance/worktrees.py status [<lease-member-or-slot>]
    python tools/maintenance/worktrees.py reconcile [--fix]
    python tools/maintenance/worktrees.py hooks install|uninstall|check
    python tools/maintenance/worktrees.py hook          (Claude Code's hooks: the event's JSON on stdin)

tools\\worktree.cmd runs it. The pool is .claude/worktrees/pool-<n>, each slot on its own branch pool-<n> with its own
warm target/ (a shared or copied target/ compiles another tree's files: never one). Nobody reaches a slot but through
this tool:

    - `acquire` takes a clean slot on today's origin/main for one piece of new work - a lease. Its holder (the
      coordinator) keeps the lease ID and the path to itself. New work is a new lease, never a reused one.
    - The holder starts each subagent with a `token`: single-use, dead two minutes after it is made. The subagent's
      first command is `join`, which makes it the slot's member and tells it the path. A slot has one member at a
      time, writing or reviewing, so the next token is made only once the member has `leave`d (or ended: the
      SubagentStop hook makes it leave) - a hand-off goes through the tool, never around it.
    - `release` gives the slot back once its work has landed: refused while a member is in it, while it is dirty or
      while it holds a commit that is on no origin branch and whose patch origin/main lacks.
    - A slot that is not clean when it would be handed out (or adopted) is set aside as needing attention.
      `status <slot>` shows what it holds; `clean <slot> --reason` throws its uncommitted changes away and frees it,
      refusing while it holds commits on no origin branch unless told `--discard-commits`.

The hooks (`hook`) hold agents to it: a subagent learns its agent ID when it starts, an edit or shell command into a
slot by anyone but its member is denied, and a subagent that stops leaves its slot. The main thread may work in a
leased slot only while nobody is a member of it. The pool is opt-in, machine by machine: `hooks install` (the
`worktree-pool-setup` skill) puts the hooks in the gitignored .claude/settings.local.json, and `acquire` refuses
until they are there.

The state is one JSON file in the repository's git common directory (shared by every worktree, never committed),
changed under a lock file beside it.
"""

import argparse
import contextlib
import datetime
import hashlib
import json
import os
import re
import secrets
import subprocess
import sys
import time

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")
sys.dont_write_bytecode = True  # no __pycache__ left in tools/maintenance

TOKEN_LIFE = datetime.timedelta(minutes=2)
STALE = datetime.timedelta(hours=6)  # a lease or member this old is listed by reconcile
LOCK_WAIT = 30.0
LOCK_STALE = 300.0  # a lock file this old is left by a killed command, and broken
LOG_KEEP = 500
POOL = os.path.join(".claude", "worktrees")
SLOT_RE = re.compile(r"\.claude[\\/]+worktrees[\\/]+(pool-\d+)(?![\w-])", re.IGNORECASE)
TOOL_CALL_RE = re.compile(r"(?:&\s*)?(?:\S*python3?(?:\.exe)?\s+)?\"?\S*?worktree(?:\.cmd|s\.py)\"?(?:\s|$)",
                          re.IGNORECASE)
SETUP_RE = re.compile(r"(?:export|cd|pushd|set-location)\s|\$env:\w+\s*=", re.IGNORECASE)
SEPARATOR_RE = re.compile(r"&&|\|\||[;|\n]")
EDIT_TOOLS = {"Edit", "Write", "NotebookEdit", "MultiEdit"}
SHELL_TOOLS = {"Bash", "PowerShell"}
WARM = ["cargo", "build", "--profile", "cards", "-p", "cantrip-script", "--bin", "corpus"]
GRANTED, REFUSED = 0, 2


class Refused(Exception):
    """A command the pool says no to; the message says why and what to do instead."""


def git(path, *args, check=True):
    result = subprocess.run(["git", *args], cwd=path, capture_output=True, text=True, encoding="utf-8",
                            errors="replace")
    if check and result.returncode != 0:
        raise Refused(f"git {' '.join(args)} failed in {path}:\n{(result.stderr or result.stdout).strip()}")
    return result.stdout.strip() if check else result


def main_checkout(start=None):
    """The repository's main working tree, which every worktree shares its .git directory with."""
    start = start or os.path.dirname(os.path.abspath(__file__))
    common = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=start,
                            capture_output=True, text=True).stdout.strip()
    if not common:
        raise SystemExit(f"{start} is not in a git repository")
    return os.path.dirname(os.path.normpath(common))


def slot_number(slot):
    return int(slot.split("-")[1])


class Pool:
    """The pool of one repository. `now`, `warm` and `hooks_ready` are replaced by the tests."""

    def __init__(self, root=None, now=None, warm=None, hooks_ready=None):
        self.root = os.path.normpath(root or main_checkout())
        dot_git = os.path.join(self.root, ".git")
        self.common = dot_git if os.path.isdir(dot_git) else os.path.join(self.root, git(self.root, "rev-parse",
                                                                                          "--git-common-dir"))
        self.state_path = os.path.join(self.common, "cantrip-worktrees.json")
        self.lock_path = self.state_path + ".lock"
        self.now = now or (lambda: datetime.datetime.now(datetime.timezone.utc))
        self.warm = warm or self._cargo_warm
        self.hooks_ready = hooks_ready or (lambda: hooks_installed(self.root))

    # --- state ---------------------------------------------------------------------------------------------------

    def path(self, slot):
        return os.path.join(self.root, POOL, slot)

    def stamp(self):
        return self.now().isoformat(timespec="seconds")

    def age(self, stamp):
        return self.now() - datetime.datetime.fromisoformat(stamp)

    def read(self):
        """The state as it stands, without the lock: writes replace the file whole, so a read sees one or the other."""
        for attempt in range(20):
            try:
                with open(self.state_path, encoding="utf-8") as f:
                    state = json.load(f)
                break
            except FileNotFoundError:
                state = {}
                break
            except (PermissionError, json.JSONDecodeError):
                if attempt == 19:
                    raise
                time.sleep(0.05)
        for key in ("slots", "leases", "members", "tokens"):
            state.setdefault(key, {})
        state.setdefault("log", [])
        return state

    def write(self, state):
        state["log"] = state["log"][-LOG_KEEP:]
        temp = f"{self.state_path}.{os.getpid()}.tmp"
        with open(temp, "w", encoding="utf-8") as f:
            json.dump(state, f, indent=1, sort_keys=True)
        for attempt in range(50):  # Windows refuses to replace a file another process has open for a moment
            try:
                os.replace(temp, self.state_path)
                return
            except PermissionError:
                if attempt == 49:
                    raise
                time.sleep(0.05)

    @contextlib.contextmanager
    def locked(self):
        """The state, held under the lock and written back when the block ends without an exception."""
        deadline = time.monotonic() + LOCK_WAIT
        while True:
            try:
                fd = os.open(self.lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
                os.write(fd, f"{os.getpid()} {self.stamp()}".encode())
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
                    raise Refused(f"the pool's lock {self.lock_path} has been held for {LOCK_WAIT:.0f}s; try again")
                time.sleep(0.1)
        try:
            state = self.read()
            yield state
            self.write(state)
        finally:
            os.remove(self.lock_path)

    def log(self, state, event, **fields):
        state["log"].append({"time": self.stamp(), "event": event, **fields})

    def purge_tokens(self, state):
        for digest, token in list(state["tokens"].items()):
            if self.age(token["made"]) > TOKEN_LIFE:
                del state["tokens"][digest]

    @staticmethod
    def member_of(state, lease_id):
        return next(((mid, m) for mid, m in state["members"].items() if m["lease"] == lease_id), (None, None))

    @staticmethod
    def lease_of(state, slot):
        return next(((lid, lease) for lid, lease in state["leases"].items() if lease["slot"] == slot), (None, None))

    def lease(self, state, lease_id):
        if lease_id not in state["leases"]:
            raise Refused(f"no lease {lease_id}: it was released, or never was ({self.status_hint()})")
        return state["leases"][lease_id]

    def member(self, state, member_id):
        if member_id not in state["members"]:
            raise Refused(f"no member {member_id}: it left, was evicted, or its agent stopped ({self.status_hint()})")
        return state["members"][member_id]

    @staticmethod
    def status_hint():
        return "tools\\worktree.cmd status lists the pool"

    # --- git -----------------------------------------------------------------------------------------------------

    def fetch(self):
        git(self.root, "fetch", "--quiet", "origin")

    def dirty(self, path):
        return [line for line in git(path, "status", "--porcelain").splitlines() if line.strip()]

    def unlanded(self, path):
        """Commits on the slot's branch that would be lost by a reset: on no origin branch, and with no equal patch
        on origin/main (a cherry-picked or rebased landing changes the hash, not the patch)."""
        lost = []
        for line in git(path, "cherry", "origin/main", "HEAD").splitlines():
            mark, sha = line.split()[:2]
            if mark == "+" and not git(path, "branch", "--remotes", "--contains", sha):
                lost.append(git(path, "log", "-1", "--format=%h %s", sha))
        return lost

    def problems(self, slot):
        """Why a slot cannot be handed out as it stands, or nothing."""
        path = self.path(slot)
        if not os.path.isdir(path):
            return ["its directory is missing"]
        found = []
        dirty = self.dirty(path)
        if dirty:
            found.append(f"{len(dirty)} uncommitted change(s): " + "; ".join(dirty[:5]))
        lost = self.unlanded(path)
        if lost:
            found.append(f"{len(lost)} unlanded commit(s): " + "; ".join(lost[:5]))
        return found

    def reset(self, slot):
        path = self.path(slot)
        git(path, "checkout", "--quiet", "-B", slot, "origin/main")

    def _cargo_warm(self, path):
        print(f"Warming {path}: {' '.join(WARM)}", flush=True)
        if subprocess.run(WARM, cwd=path).returncode != 0:
            raise Refused(f"the warm build failed in {path}; the slot is not registered")

    # --- commands ------------------------------------------------------------------------------------------------

    def create(self, count=1):
        self.fetch()
        made = []
        for _ in range(count):
            with self.locked() as state:
                pool_dir = os.path.join(self.root, POOL)
                taken = set(state["slots"]) | {d for d in (os.listdir(pool_dir) if os.path.isdir(pool_dir) else [])
                                               if re.fullmatch(r"pool-\d+", d)}
                n = max((slot_number(s) for s in taken), default=0) + 1
                slot = f"pool-{n}"
                state["slots"][slot] = {"state": "creating", "note": ""}
            path = self.path(slot)
            try:
                git(self.root, "worktree", "add", "--quiet", "-B", slot, path, "origin/main")
                self.warm(path)
            except Refused:
                with self.locked() as state:
                    state["slots"][slot] = {"state": "attention", "note": "creation failed"}
                raise
            with self.locked() as state:
                state["slots"][slot] = {"state": "free", "note": ""}
                self.log(state, "create", slot=slot)
            made.append(slot)
        return "\n".join(f"Created {s}: free, warm, on origin/main." for s in made)

    def acquire(self, purpose, name=None):
        if not self.hooks_ready():
            raise Refused("the pool's hooks are not installed on this machine, so its agents cannot join a worktree "
                          "and nothing keeps them out of one. Managing worktrees through the pool is opt-in: the "
                          "`worktree-pool-setup` skill (`tools\\worktree.cmd hooks install`, then a restart) sets "
                          "it up.")
        self.fetch()
        with self.locked() as state:
            skipped = []
            for slot in sorted(state["slots"], key=slot_number):
                if state["slots"][slot]["state"] != "free":
                    continue
                found = self.problems(slot)
                if not found:
                    try:
                        self.reset(slot)
                    except Refused as e:
                        found = [str(e)]
                if found:
                    state["slots"][slot] = {"state": "attention", "note": "; ".join(found)}
                    self.log(state, "attention", slot=slot, note="; ".join(found))
                    skipped.append(f"{slot}: {'; '.join(found)}")
                    continue
                lease_id = "L-" + secrets.token_urlsafe(12)
                state["slots"][slot] = {"state": "leased", "note": ""}
                state["leases"][lease_id] = {"slot": slot, "purpose": purpose, "name": name or "",
                                             "acquired": self.stamp()}
                self.log(state, "acquire", slot=slot, lease=lease_id, purpose=purpose)
                return holder_brief(lease_id, slot, self.path(slot), purpose)
            table = self.pool_table(state)
        # Refused once the lock's block has written the slots it set aside.
        raise Refused("NO WORKTREE AVAILABLE.\n" + table
                      + ("\nSet aside as needing attention just now:\n  " + "\n  ".join(skipped) if skipped else "")
                      + "\nWait for a lease to be released, or grow the pool with `tools\\worktree.cmd create`"
                        " (a human's call: each slot's builds take every core).")

    def token(self, lease_id, read_only=False, name=None):
        with self.locked() as state:
            lease = self.lease(state, lease_id)
            self.purge_tokens(state)
            mid, member = self.member_of(state, lease_id)
            if member:
                raise Refused(f"{lease['slot']} has a member, {member['name'] or mid} (agent {member['agent']}), "
                              f"since {member['joined']}. One member at a time: the next token is made once it has "
                              f"left (`leave`, or its agent stopping), or after `evict` once you know its agent has "
                              f"ended.")
            if any(t["lease"] == lease_id for t in state["tokens"].values()):
                raise Refused(f"a token for {lease['slot']} is already out and unused; it dies "
                              f"{TOKEN_LIFE.seconds // 60} minutes after it was made. Start the agent that has it, "
                              f"or wait for it to die.")
            token = "T-" + secrets.token_urlsafe(12)
            state["tokens"][digest(token)] = {"lease": lease_id, "read_only": read_only, "name": name or "",
                                              "made": self.stamp()}
            self.log(state, "token", lease=lease_id, read_only=read_only, name=name or "")
        kind = "read-only " if read_only else ""
        return (f"TOKEN {token}\n"
                f"A single-use {kind}token for one subagent, dead in {TOKEN_LIFE.seconds // 60} minutes if unused. "
                f"Start that subagent now, and put this in its prompt, word for word:\n\n"
                f"  Your workspace is a pool worktree. Your first action, before anything else: run\n"
                f"  `tools\\worktree.cmd join {token} --agent-id <your agent ID>` from {self.root} (your agent ID\n"
                f"  is in your context, from the SubagentStart hook) and follow what it prints.\n\n"
                f"Never give the agent the path or the lease ID.")

    def join(self, token, agent_id):
        if not agent_id:
            raise Refused("join needs --agent-id: your agent ID is in your context from the SubagentStart hook "
                          "(\"Your agent ID is ...\"). Without one, the hooks are not set up: tell whoever started "
                          "you.")
        with self.locked() as state:
            entry = state["tokens"].pop(digest(token), None)
            if entry is None:
                raise Refused("that token is not valid: tokens are single-use. Ask whoever started you for a new one;"
                              " do not work in any worktree meanwhile.")
            if self.age(entry["made"]) > TOKEN_LIFE:
                raise Refused(f"that token died {TOKEN_LIFE.seconds // 60} minutes after it was made. Ask whoever "
                              f"started you for a new one; do not work in any worktree meanwhile.")
            lease = self.lease(state, entry["lease"])
            mid, member = self.member_of(state, entry["lease"])
            if member:
                raise Refused(f"{lease['slot']} already has a member, {member['name'] or mid}. Tell whoever started "
                              f"you.")
            member_id = "M-" + secrets.token_urlsafe(12)
            state["members"][member_id] = {"lease": entry["lease"], "agent": agent_id, "name": entry["name"],
                                           "read_only": entry["read_only"], "joined": self.stamp()}
            self.log(state, "join", lease=entry["lease"], member=member_id, agent=agent_id)
        return member_brief(member_id, lease["slot"], self.path(lease["slot"]), entry["read_only"])

    def leave(self, member_id):
        with self.locked() as state:
            member = self.member(state, member_id)
            slot = state["leases"][member["lease"]]["slot"]
            del state["members"][member_id]
            self.log(state, "leave", lease=member["lease"], member=member_id)
        return (f"Left {slot}. Its files are unchanged; what you did not commit stays for its holder. The tree as you "
                f"left it:\n" + self.tree_report(slot))

    def leave_agent(self, agent_id, why):
        """Every membership an agent holds ends: it stopped."""
        with self.locked() as state:
            for mid, member in list(state["members"].items()):
                if member["agent"] == agent_id:
                    del state["members"][mid]
                    self.log(state, "leave", lease=member["lease"], member=mid, why=why)

    def evict(self, lease_id, member_id, reason):
        with self.locked() as state:
            self.lease(state, lease_id)
            member = self.member(state, member_id)
            if member["lease"] != lease_id:
                raise Refused(f"{member_id} is not a member of lease {lease_id}")
            del state["members"][member_id]
            self.log(state, "evict", lease=lease_id, member=member_id, reason=reason)
        return f"Evicted {member_id}. Files are as it left them."

    def sync(self, some_id):
        with self.locked() as state:
            if some_id in state["members"]:
                slot = state["leases"][state["members"][some_id]["lease"]]["slot"]
            else:
                lease = self.lease(state, some_id)
                mid, member = self.member_of(state, some_id)
                if member:
                    raise Refused(f"{lease['slot']} has a member, {member['name'] or mid}: only it may sync now")
                slot = lease["slot"]
            path = self.path(slot)
            dirty = self.dirty(path)
            if dirty:
                raise Refused(f"{slot} has uncommitted changes; commit or drop them first:\n  " + "\n  ".join(dirty))
            self.fetch()
            result = git(path, "rebase", "origin/main", check=False)
            if result.returncode != 0:
                return (f"The rebase of {slot} onto origin/main stopped on a conflict and is still in progress:\n"
                        f"{(result.stdout + result.stderr).strip()}\nResolve it and `git rebase --continue`, or "
                        f"`git rebase --abort`.")
        return f"{slot} is rebased onto origin/main ({git(path, 'log', '-1', '--format=%h %s', 'origin/main')})."

    def release(self, lease_id, discard=False, reason=None):
        if discard and not reason:
            raise Refused("--discard needs --reason: whatever the slot holds is thrown away, and the log says why")
        self.fetch()
        with self.locked() as state:
            lease = self.lease(state, lease_id)
            slot = lease["slot"]
            mid, member = self.member_of(state, lease_id)
            if member:
                raise Refused(f"{slot} still has a member, {member['name'] or mid} (agent {member['agent']}). It "
                              f"leaves when done or when its agent stops; `evict` it only once you know its agent "
                              f"has ended.")
            path = self.path(slot)
            found = self.problems(slot) if os.path.isdir(path) else []
            if found and not discard:
                raise Refused(f"{slot} holds work that is not on origin:\n  " + "\n  ".join(found)
                              + "\nLand it (or push it to a branch) and release again; only if it is truly unwanted, "
                                "`release --discard --reason \"...\"`.")
            try:
                if discard:
                    git(path, "reset", "--quiet", "--hard")
                    git(path, "clean", "--quiet", "-fd")  # untracked files, never the ignored target/
                self.reset(slot)
            except Refused as e:
                state["slots"][slot] = {"state": "attention", "note": f"reset failed on release: {e}"}
                del state["leases"][lease_id]
                self.log(state, "release", slot=slot, lease=lease_id, failed=str(e))
                return (f"Released lease {lease_id}, but {slot} could not be reset (a process may still hold its "
                        f"files) and needs attention:\n{e}")
            state["slots"][slot] = {"state": "free", "note": ""}
            del state["leases"][lease_id]
            self.log(state, "release", slot=slot, lease=lease_id, discarded="; ".join(found) if discard else "",
                     reason=reason or "")
        return f"Released {slot}: free, on origin/main."

    def clean(self, slot, reason, discard_commits=False):
        """An unleased slot set aside as needing attention, emptied and put back in the pool: nobody can work in it
        to clear it (the hooks keep everyone out of a slot nobody holds), so the tool does."""
        if not reason:
            raise Refused("clean needs --reason: whatever the slot holds is thrown away, and the log says why")
        self.fetch()
        with self.locked() as state:
            if slot not in state["slots"]:
                raise Refused(f"{slot} is not in the pool ({self.status_hint()})")
            lid, lease = self.lease_of(state, slot)
            if lease:
                raise Refused(f"{slot} is held by lease {lid} ({lease['purpose']}); its holder releases it")
            path = self.path(slot)
            if not os.path.isdir(path):
                raise Refused(f"{slot}'s directory is missing")
            dirty, lost = self.dirty(path), self.unlanded(path)
            if lost and not discard_commits:
                raise Refused(f"{slot} holds {len(lost)} commit(s) on no origin branch:\n  " + "\n  ".join(lost)
                              + f"\nPush them to a branch (`git -C <slot> push origin {slot}:refs/heads/<name>`, "
                                f"by whoever may) and clean again; only if they are truly unwanted, "
                                f"`--discard-commits`.")
            git(path, "reset", "--quiet", "--hard")
            git(path, "clean", "--quiet", "-fd")  # untracked files, never the ignored target/
            self.reset(slot)
            state["slots"][slot] = {"state": "free", "note": ""}
            self.log(state, "clean", slot=slot, discarded="; ".join(dirty + lost), reason=reason)
        return (f"Cleaned {slot}: free, on origin/main (threw away {len(dirty)} uncommitted change(s) and "
                f"{len(lost)} unlanded commit(s)).")

    def status(self, some_id=None):
        state = self.read()
        if some_id in state["slots"] and not self.lease_of(state, some_id)[1]:
            entry = state["slots"][some_id]
            return (f"{some_id}: {entry['state']}, not leased" + (f" - {entry['note']}" if entry["note"] else "")
                    + "\n" + self.tree_report(some_id))
        if some_id in state["slots"]:
            some_id = self.lease_of(state, some_id)[0]
        if some_id:
            if some_id in state["members"]:
                some_id = state["members"][some_id]["lease"]
            lease = self.lease(state, some_id)
            mid, member = self.member_of(state, some_id)
            text = (f"Lease {some_id}: {lease['slot']} at {self.path(lease['slot'])}\n"
                    f"  for: {lease['purpose']}  since {lease['acquired']}\n"
                    f"  member: {self.member_line(mid, member) if member else 'none'}\n")
            return text + self.tree_report(lease["slot"])
        return self.pool_table(state)

    def reconcile(self, fix=False):
        out = []
        with self.locked() as state:
            listed = {}
            for block in git(self.root, "worktree", "list", "--porcelain").split("\n\n"):
                fields = dict(line.split(" ", 1) for line in block.splitlines() if " " in line)
                if "worktree" in fields:
                    listed[os.path.normcase(os.path.normpath(fields["worktree"]))] = fields.get("branch", "")
            pool_dir = os.path.join(self.root, POOL)
            dirs = sorted((d for d in (os.listdir(pool_dir) if os.path.isdir(pool_dir) else [])
                           if os.path.isdir(os.path.join(pool_dir, d))), key=lambda d: (not d.startswith("pool-"), d))
            for d in dirs:
                path = os.path.join(pool_dir, d)
                known = os.path.normcase(os.path.normpath(path)) in listed
                if re.fullmatch(r"pool-\d+", d):
                    if d not in state["slots"]:
                        if not known:
                            out.append(f"{d}: a directory that is not a git worktree; left alone")
                            continue
                        found = self.problems(d)
                        out.append(f"{d}: not in the pool" + (f" ({'; '.join(found)})" if found else " (clean)"))
                        if fix:
                            state["slots"][d] = ({"state": "attention", "note": "; ".join(found)} if found
                                                 else {"state": "free", "note": ""})
                            self.log(state, "adopt", slot=d, note="; ".join(found))
                            out.append(f"  -> registered as {state['slots'][d]['state']}")
                else:
                    kind = "the Agent tool's isolation worktree" if d.startswith("agent-") else "not a pool slot"
                    where = f"on {listed[os.path.normcase(os.path.normpath(path))]}" if known else "not a git worktree"
                    out.append(f"{d}: {kind}, {where}; not managed here")
            for slot, entry in sorted(state["slots"].items(), key=lambda s: slot_number(s[0])):
                if not os.path.isdir(self.path(slot)) and entry["state"] != "missing":
                    out.append(f"{slot}: registered, but its directory is missing")
                    if fix:
                        entry.update(state="missing", note="directory missing")
                elif entry["state"] == "attention":
                    found = self.problems(slot)
                    out.append(f"{slot}: needs attention: {'; '.join(found) or entry['note'] or 'now clean'}")
                    if fix and not found and not self.lease_of(state, slot)[1]:
                        entry.update(state="free", note="")
                        out.append("  -> clean again: free")
            for lid, lease in state["leases"].items():
                mid, member = self.member_of(state, lid)
                if self.age(lease["acquired"]) > STALE:
                    out.append(f"{lease['slot']}: leased {lease['acquired']} for {lease['purpose']} - over "
                               f"{STALE.seconds // 3600 + STALE.days * 24}h; is its holder still at work?\n"
                               + indent(self.tree_report(lease["slot"])))
                if member and self.age(member["joined"]) > STALE:
                    out.append(f"{lease['slot']}: member {self.member_line(mid, member)} - is its agent still "
                               f"running? If it has ended, the holder `evict`s it.")
            expired = [t for t in state["tokens"].values() if self.age(t["made"]) > TOKEN_LIFE]
            if expired:
                out.append(f"{len(expired)} expired token(s)" + (" -> dropped" if fix else ""))
                if fix:
                    self.purge_tokens(state)
        return "\n".join(out) if out else "The pool and git agree; nothing stale."

    # --- reports -------------------------------------------------------------------------------------------------

    def member_line(self, mid, member):
        kind = "read-only" if member["read_only"] else "writing"
        # The ID is what `evict --member` takes; a name alone sends a coordinator looking for it.
        label = f"{member['name']} {mid}" if member["name"] else mid
        return f"{label} ({kind}, agent {member['agent']}, since {member['joined']})"

    def pool_table(self, state):
        self.purge_tokens(state)
        if not state["slots"]:
            return "The pool has no slots (`tools\\worktree.cmd reconcile --fix` adopts existing pool-<n> directories)."
        lines = []
        for slot, entry in sorted(state["slots"].items(), key=lambda s: slot_number(s[0])):
            lid, lease = self.lease_of(state, slot)
            line = f"{slot}: {entry['state']}"
            if lease:
                mid, member = self.member_of(state, lid)
                line += f" - {lease['purpose']}{' (' + lease['name'] + ')' if lease['name'] else ''}, since " \
                        f"{lease['acquired']}; member: {self.member_line(mid, member) if member else 'none'}"
                if any(t["lease"] == lid for t in state["tokens"].values()):
                    line += "; a token is out"
            elif entry["note"]:
                line += f" - {entry['note']}"
            lines.append(line)
        return "\n".join(lines)

    def tree_report(self, slot):
        path = self.path(slot)
        if not os.path.isdir(path):
            return "  (its directory is missing)"
        head = git(path, "log", "-1", "--format=%h %s (%cr)")
        ahead = git(path, "log", "--format=%h %s", "origin/main..HEAD").splitlines()
        dirty = self.dirty(path)
        return (f"  HEAD {head}\n"
                f"  commits not on origin/main: {len(ahead)}" + "".join(f"\n    {c}" for c in ahead[:10]) + "\n"
                f"  uncommitted changes: {len(dirty)}" + "".join(f"\n    {d}" for d in dirty[:10]))

    # --- hooks ---------------------------------------------------------------------------------------------------

    def hook(self, event):
        """Claude Code's hook events: the answer to print, or None."""
        name = event.get("hook_event_name")
        agent = event.get("agent_id")
        if name == "SubagentStart" and agent:
            return {"hookSpecificOutput": {"hookEventName": "SubagentStart", "additionalContext":
                    f"Your agent ID is {agent}. A pool worktree is reached only through `tools\\worktree.cmd join "
                    f"<token> --agent-id {agent}`, with a token from whoever started you."}}
        if name == "SubagentStop" and agent:
            if any(m["agent"] == agent for m in self.read()["members"].values()):
                self.leave_agent(agent, "its agent stopped")
            return None
        if name != "PreToolUse" or not os.path.exists(self.state_path):
            return None
        reason = self.denial(self.read(), event.get("tool_name"), event.get("tool_input") or {},
                             event.get("cwd") or "", agent)
        if reason:
            return {"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                                           "permissionDecisionReason": reason}}
        return None

    def slots_touched(self, tool, tool_input, cwd):
        if tool in EDIT_TOOLS:
            paths = [tool_input.get("file_path") or tool_input.get("notebook_path") or ""]
            return {m.group(1).lower() for p in paths for m in [SLOT_RE.search(p)] if m and self.ours(p)}
        if tool in SHELL_TOOLS:
            command = tool_input.get("command") or ""
            touched = {m.group(1).lower() for m in SLOT_RE.finditer(command)}
            m = SLOT_RE.search(cwd)
            if m and self.ours(cwd) and not runs_only_the_tool(command):  # the pool's own commands: anyone may ask
                touched.add(m.group(1).lower())
            return touched
        return set()

    def ours(self, path):
        """Whether an absolute path is under this repository's pool directory (a relative one is taken as it is)."""
        if not os.path.isabs(path):
            return True
        pool = os.path.normcase(os.path.join(self.root, POOL))
        return os.path.normcase(os.path.normpath(path)).startswith(pool + os.sep)

    def denial(self, state, tool, tool_input, cwd, agent):
        for slot in sorted(self.slots_touched(tool, tool_input, cwd)):
            if slot not in state["slots"]:
                continue  # not in the pool (yet): reconcile adopts it
            lid, lease = self.lease_of(state, slot)
            if not lease:
                return (f"{slot} is a pool worktree nobody holds. Work in the pool starts with `tools\\worktree.cmd "
                        f"acquire` (or a token from whoever holds it); never by its path.")
            mid, member = self.member_of(state, lid)
            if agent is None:
                if member:
                    return (f"{slot} has a member at work, {self.member_line(mid, member)}; the main thread stays out "
                            f"of it until the member leaves.")
                continue
            if not member or member["agent"] != agent:
                return (f"{slot} is a pool worktree you are not a member of (your agent ID is {agent}). Join it with "
                        f"`tools\\worktree.cmd join <token> --agent-id {agent}`, the token from whoever started you; "
                        f"never work in a worktree by a path you were given.")
            if member["read_only"] and tool in EDIT_TOOLS:
                return f"You joined {slot} read-only: you change no files in it."
        return None


HOOK_COMMAND = 'python "$CLAUDE_PROJECT_DIR/tools/maintenance/worktrees.py" hook'
HOOK_EVENTS = {"SubagentStart": None, "PreToolUse": "Edit|Write|MultiEdit|NotebookEdit|Bash|PowerShell",
               "SubagentStop": None}


def settings_path(root):
    """This machine's own Claude Code settings for the repository: gitignored, never committed."""
    return os.path.join(root, ".claude", "settings.local.json")


def ours(entry):
    return any("worktrees.py" in h.get("command", "") and h.get("command", "").endswith(" hook")
               for h in entry.get("hooks", []))


def runs_only_the_tool(command):
    """Whether a shell command does nothing but run this tool (after setting a variable or a directory): only then
    may it run from inside a slot that is not the caller's. Naming the tool anywhere in it is not enough."""
    if re.search(r"\$\(|`|[<>]", command):
        return False  # a substitution or redirection runs or writes something else
    parts = [p.strip() for p in SEPARATOR_RE.split(command) if p.strip()]
    tool = [p for p in parts if TOOL_CALL_RE.match(p)]
    return bool(tool) and all(p in tool or SETUP_RE.match(p) for p in parts)


def hooks_installed(root):
    try:
        with open(settings_path(root), encoding="utf-8-sig") as f:
            hooks = json.load(f).get("hooks", {})
    except (FileNotFoundError, json.JSONDecodeError):
        return False
    return all(any(ours(e) for e in hooks.get(event, [])) for event in HOOK_EVENTS)


def hooks_command(root, action):
    """The opt-in: the pool's hooks in this machine's .claude/settings.local.json, added or taken out."""
    path = settings_path(root)
    if action == "check":
        return f"The pool's hooks are {'installed' if hooks_installed(root) else 'not installed'} in {path}."
    try:
        with open(path, encoding="utf-8-sig") as f:
            settings = json.load(f)
    except FileNotFoundError:
        settings = {}
    hooks = settings.setdefault("hooks", {})
    for event, matcher in HOOK_EVENTS.items():
        kept = [e for e in hooks.get(event, []) if not ours(e)]
        if action == "install":
            entry = {"hooks": [{"type": "command", "command": HOOK_COMMAND}]}
            if matcher:
                entry = {"matcher": matcher, **entry}
            kept.append(entry)
        if kept:
            hooks[event] = kept
        else:
            hooks.pop(event, None)
    if not hooks:
        settings.pop("hooks")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(settings, f, indent=2)
        f.write("\n")
    verb = "Installed" if action == "install" else "Removed"
    return (f"{verb} the pool's hooks in {path}. Claude Code reads hooks when a session starts: restart it (or "
            f"review them in /hooks) before the pool is used.")


def digest(token):
    return hashlib.sha256(token.encode()).hexdigest()


def indent(text):
    return "\n".join("  " + line for line in text.splitlines())


def holder_brief(lease_id, slot, path, purpose):
    return f"""GRANTED {slot} for: {purpose}
LEASE {lease_id}
PATH {path}

You hold this worktree. Its path and the lease ID are yours alone: never put either in a prompt or a message.
- Every agent that works in it is started with its own token: `tools\\worktree.cmd token {lease_id}`
  (`--read-only` for an agent that only reads - not a card reviewer, which breaks the card and restores it - and `--name` to label it). The token is single-use and dies in {TOKEN_LIFE.seconds // 60} minutes;
  the agent's first command is `join`, and the token's output says what to put in its prompt.
- One member at a time. The next token is made only once the current member has left - it runs `leave`, or its
  agent stops (the hook makes it leave). An agent resumed with SendMessage to work in the tree again gets a fresh
  token in that message. `evict` only a member whose agent you know has ended.
- You may work in the tree yourself only while it has no member.
- `tools\\worktree.cmd sync {lease_id}` rebases it onto origin/main when no member is in it.
- When the work has landed: `tools\\worktree.cmd release {lease_id}` - the last thing done with it. New work takes a
  new `acquire`, never this lease.
- Lost track? `tools\\worktree.cmd status` lists every lease, member and token."""


def member_brief(member_id, slot, path, read_only):
    mode = ("READ-ONLY: change no files, check out nothing, clean nothing; builds and test runs are fine."
            if read_only else "You may change, build and commit here.")
    return f"""GRANTED {slot}
MEMBER {member_id}
PATH {path}

{mode}
- Work only here: start every shell command with `cd {path}`, and keep every file path under it. Builds stay in its
  own target/ (never CARGO_TARGET_DIR or --target-dir elsewhere).
- Never pass the path or your member ID to anyone, and never start a subagent to work in this tree.
- Commit before you finish (and push only where your instructions say to). Your last command, as you finish:
  `tools\\worktree.cmd leave {member_id}`. Leaving changes no files: what you did not commit stays in the tree for its
  holder, who may throw it away. Stopping ends your membership too; to work here again you need a new token.
- `tools\\worktree.cmd sync {member_id}` rebases the tree onto origin/main if your instructions call for it."""


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("create", help="add warm slots to the pool")
    p.add_argument("--count", type=int, default=1)
    p.add_argument("--no-warm", action="store_true", help="skip the warm build")
    p = sub.add_parser("acquire", help="take a clean slot for new work")
    p.add_argument("--for", dest="purpose", required=True, help="what the work is, e.g. \"card: Lightning Bolt\"")
    p.add_argument("--name", help="who holds it, for people reading status")
    p = sub.add_parser("token", help="a single-use token that lets one subagent join a lease's slot")
    p.add_argument("lease_id")
    p.add_argument("--read-only", action="store_true")
    p.add_argument("--name", help="the agent it is for, e.g. \"review: Lightning Bolt\"")
    p = sub.add_parser("join", help="redeem a token: the slot's path")
    p.add_argument("token")
    p.add_argument("--agent-id", help="your agent ID, from the SubagentStart hook")
    p = sub.add_parser("leave", help="end your membership")
    p.add_argument("member_id")
    p = sub.add_parser("sync", help="rebase the slot onto origin/main")
    p.add_argument("id", help="a member ID, or the lease ID while the slot has no member")
    p = sub.add_parser("evict", help="end the membership of an agent known to have ended")
    p.add_argument("lease_id")
    p.add_argument("--member", required=True)
    p.add_argument("--reason", required=True)
    p = sub.add_parser("release", help="give the slot back once its work has landed")
    p.add_argument("lease_id")
    p.add_argument("--discard", action="store_true", help="throw away unlanded work (needs --reason)")
    p.add_argument("--reason")
    p = sub.add_parser("clean", help="empty an unleased slot set aside as needing attention, and free it")
    p.add_argument("slot", help="e.g. pool-1")
    p.add_argument("--reason", required=True)
    p.add_argument("--discard-commits", action="store_true", help="also throw away commits on no origin branch")
    p = sub.add_parser("status", help="the pool, or one lease, member or slot")
    p.add_argument("id", nargs="?")
    p = sub.add_parser("reconcile", help="compare the pool with git; find what is stale")
    p.add_argument("--fix", action="store_true", help="adopt pool directories, drop dead tokens (never a lease)")
    sub.add_parser("hook", help="Claude Code's hooks, the event on stdin")
    p = sub.add_parser("hooks", help="opt in or out: the hooks in this machine's .claude/settings.local.json")
    p.add_argument("action", choices=["install", "uninstall", "check"])
    args = parser.parse_args()

    if args.command == "hooks":
        print(hooks_command(main_checkout(), args.action))
        return GRANTED

    if args.command == "hook":
        # A hook never stands in the way by failing: a broken state file or git lets the call through.
        try:
            start = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
            root = start if os.path.isdir(os.path.join(start, ".git")) else main_checkout(start)
            answer = Pool(root=root).hook(json.loads(sys.stdin.buffer.read().decode("utf-8-sig")))
        except Exception as e:  # noqa: BLE001
            print(f"worktree hook: {e}", file=sys.stderr)
            return 0
        if answer:
            print(json.dumps(answer))
        return 0

    pool = Pool(warm=(lambda path: None) if getattr(args, "no_warm", False) else None)
    try:
        out = {
            "create": lambda: pool.create(args.count),
            "acquire": lambda: pool.acquire(args.purpose, args.name),
            "token": lambda: pool.token(args.lease_id, args.read_only, args.name),
            "join": lambda: pool.join(args.token, args.agent_id),
            "leave": lambda: pool.leave(args.member_id),
            "sync": lambda: pool.sync(args.id),
            "evict": lambda: pool.evict(args.lease_id, args.member, args.reason),
            "release": lambda: pool.release(args.lease_id, args.discard, args.reason),
            "clean": lambda: pool.clean(args.slot, args.reason, args.discard_commits),
            "status": lambda: pool.status(args.id),
            "reconcile": lambda: pool.reconcile(args.fix),
        }[args.command]()
    except Refused as e:
        print(f"REFUSED: {e}" if not str(e).startswith("NO WORKTREE") else str(e))
        return REFUSED
    print(out)
    return GRANTED


if __name__ == "__main__":
    sys.exit(main())
