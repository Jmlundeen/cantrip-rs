"""Wait for pull requests on cantrip-rs/cantrip-rs, and clear the queue with a fresh Claude Code session each time.

    python tools/maintenance/pr-watch.py [--interval 60] [--once] [--mark-seen] [--include-drafts] [--dry-run]

tools\\pr-watch.cmd runs it. Every --interval seconds it asks GitHub's REST API for the open pull requests (`gh api`), with
the last answer's ETag: an unchanged list comes back 304 Not Modified, which costs nothing against the rate limit. A pull
request is new work when its head commit is one no session has been started for (a new PR, or new commits pushed to
one). New work starts `claude -p` in the repository root - a new session, so a new context each time - whose prompt has
it clear the whole queue through the `review-card-pr` skill, two review agents at a time, and stop. When it exits,
every open pull request's head is marked seen and the wait begins again; a PR the session left open (one waiting on a
human's decision) is not taken up again until it gets new commits.

While a session runs the watcher prints its progress per pull request, from two sources. What it sees itself, every
--interval seconds: a slot of the worktree pool taken or given back for a PR, an agent going to work in it or leaving
(`worktree.cmd status`), and a PR closed or opened (GitHub). And what the session says: it streams its events
(`--output-format stream-json`), and the watcher prints, as they come, the `PR #N:` lines its prompt has it write at
each step, then its final report. Every event is written to .claude/pr-watch/<time>.jsonl (gitignored), and what has
been seen to .claude/pr-watch/seen.json.

A terminated watcher loses no work. The session's ID is kept in .claude/pr-watch/session.json until the session
reports; the next watcher finds it there and resumes that session (`claude -p --resume`), with a prompt that has it
recover each slot it holds - the dead member evicted, nothing in the slot discarded - and send each of its agents back
to work by SendMessage, context kept, to finish the step it was on. A session that will not resume (three tries) is
dropped, and the new session it is replaced by adopts the slots it left: every commit in them kept, reviewed afresh.
--mark-seen marks today's open pull requests seen without starting a session, so only
later work wakes it; --once stops after one session; --dry-run prints the command instead of running it. Only one
watcher runs at a time (.claude/pr-watch/lock).
"""

import argparse
import datetime
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
import uuid
from pathlib import Path

REPO = "cantrip-rs/cantrip-rs"
API = f"repos/{REPO}/pulls?state=open&per_page=100"
ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / ".claude" / "pr-watch"
SEEN = STATE / "seen.json"
SESSION = STATE / "session.json"  # the session running now, or one a terminated watcher left; gone once it reports
MAX_RESUMES = 3  # a session that stops this many times running without a report is given up, and a new one adopts its slots

PROMPT = f"""Clear the pull-request queue of {REPO}.

List the open pull requests (`gh pr list --repo {REPO} --state open`) and land each one through the `review-card-pr`
skill and `add-cards/coordinator.md`, oldest first, with at most two review agents working at a time: a lease and a
cherry-pick, an Opus review agent, a Sonnet fix agent for findings that change code, the reviewer's confirmation, then
the landing (the test run it names, fetch and rebase, push, `card-issues.py log N landed` where the card has an issue,
`gh pr comment` with the hashes and what the review found, `gh pr close`, release the lease). A pull request that adds
no card still gets a review agent, against CLAUDE.md. List the queue again after each landing: new pull requests that
arrive while you work are part of it.

Before taking a new lease, read `tools\\worktree.cmd status`: a slot leased "for PR #N" whose pull request is still open
was left by a session that was stopped (its lease ID is in `status pool-<n>`). Adopt it rather than starting that PR
again: evict its member if one is listed (its agent ended with the session), restore a reviewer's temporary break edit
(an uncommitted change to a card's code that no commit explains), keep every commit in the slot - the PR's, a
reviewer's fixes, a fix agent's - and send that slot's commits to a fresh review agent, telling it which commits are
the PR's and which were made on top. A slot leased for a pull request that is closed is released.

A pull request that needs a decision only the user can make (a rules exception, a cherry-pick conflict, a finding you
cannot settle) is left open and unlanded, its lease released with --discard, and named in your report; carry on with
the rest.

Report progress as you go: each time a pull request moves on (lease taken, review started, review verdict, fix agent
started, fixes confirmed, tests passed or failed, landed, left open), write a line of its own, on its own, that starts
`PR #<N>:` and says it in a few words - for example `PR #741: review blocked, 2 findings; fix agent started`. The
watcher prints those lines to whoever is watching; nothing else you write reaches them until the end.

Do not start any wait, poll or sleep loop for new pull requests: something else does that. Do not end your turn while a
subagent or a test run you started is still going - wait for its notification. When no open pull request is left that
you can land, finish with a short report per pull request: what landed and its hashes, what the reviews found and
changed, and anything left for the user."""

RESUME = f"""You were stopped part way through clearing the pull-request queue of {REPO}: the watcher that runs you was
terminated, and every subagent you had running was killed with it. This is the same session, resumed; carry on from
where you were, without starting any pull request over.

First recover each lease you hold (`tools\\worktree.cmd status`, your lease IDs are above in this conversation):
- a member still listed is dead: `evict` it, naming the interruption as the reason;
- look at the slot's uncommitted changes: a review agent's temporary break edit is left for that agent to restore, a fix
  agent's unfinished work for that agent to finish - discard nothing yourself;
- continue the slot's agent where it stopped: SendMessage to the same agent ID (it keeps its context) with a fresh
  token from `worktree.cmd token <lease>`, saying it was interrupted, that it rejoins with that token, restores any
  temporary break edit first, and redoes the step it was on - a test or build run it started did not finish;
- a slot whose agent had already reported goes on to its next step as it would have.
Then go on clearing the queue exactly as your first instructions say, listing it again, and finish with the same
report. Keep writing a `PR #<N>:` line at each step, starting with one per PR saying it was resumed."""


class Poller:
    """The open pull requests, asked through `gh api` (its login, and a TLS stack that trusts what the system does) with
    the last ETag, so an unchanged list - 304 Not Modified - costs no rate limit."""

    def __init__(self):
        self.gh = shutil.which("gh") or sys.exit("pr-watch: gh is not on PATH (https://cli.github.com, then gh auth login)")
        self.etag = None
        self.prs = []

    def poll(self, fresh=False):
        cmd = [self.gh, "api", "-i", API]
        if self.etag and not fresh:
            cmd += ["-H", f"If-None-Match: {self.etag}"]
        out = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
        head, _, body = out.stdout.partition("\n\n")  # text mode has already made \r\n into \n
        status = head.split("\n", 1)[0]
        if " 304" in status:
            return self.prs
        if out.returncode != 0:
            raise OSError((out.stderr or status).strip())
        for line in head.split("\n")[1:]:
            name, _, value = line.partition(":")
            if name.strip().lower() == "etag":
                self.etag = value.strip()
        self.prs = [
            {"number": p["number"], "sha": p["head"]["sha"], "title": p["title"], "author": p["user"]["login"], "draft": p.get("draft", False)}
            for p in json.loads(body)
        ]
        return self.prs


def load_seen():
    try:
        return json.loads(SEEN.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def save_seen(prs):
    SEEN.write_text(json.dumps({str(p["number"]): p["sha"] for p in prs}, indent=1), encoding="utf-8")


def lock():
    """Hold .claude/pr-watch/lock for the life of the process, or exit if another watcher holds it."""
    f = open(STATE / "lock", "a+")
    try:
        if os.name == "nt":
            import msvcrt
            msvcrt.locking(f.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        sys.exit("pr-watch: another watcher is already running")
    return f


def stamp():
    return datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")


def progress(event):
    """The `PR #N:` lines of a stream-json event the session itself wrote (not one of its subagents), or its final
    report."""
    if event.get("type") == "result":
        return [], event.get("result") or ""
    if event.get("type") != "assistant" or event.get("parent_tool_use_id"):
        return [], None
    text = "\n".join(b.get("text", "") for b in event.get("message", {}).get("content", []) if b.get("type") == "text")
    return [line.strip() for line in text.splitlines() if line.strip().startswith("PR #")], None


def pool():
    """What each leased slot of the worktree pool is for and who is at work in it: {slot: (purpose, member or None)}."""
    out = subprocess.run([sys.executable, str(ROOT / "tools" / "maintenance" / "worktrees.py"), "status"],
                         cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace").stdout
    slots = {}
    for line in out.splitlines():
        m = re.match(r"(pool-\d+): leased - (.+?), since [^;]+; member: (.*)$", line.strip())
        if m:
            member = m[3]  # "none", "M-... (writing, ...)" or "<name> M-... (writing, ...)"
            slots[m[1]] = (m[2], None if member == "none" else member.split(" M-")[0] if " M-" in member else "an agent")
    return slots


def pool_changes(before, after):
    for slot in sorted(set(before) | set(after)):
        b, a = before.get(slot), after.get(slot)
        if b == a:
            continue
        if b and (not a or a[0] != b[0]):
            yield f"{b[0]}: gave back {slot}"
        if a and (not b or b[0] != a[0]):
            yield f"{a[0]}: took {slot}" + (f", {a[1]} at work" if a[1] else "")
        elif a and a[1] != b[1]:
            yield f"{a[0]}: " + (f"{a[1]} at work in {slot}" if a[1] else f"{b[1]} done in {slot}")


def pr_changes(before, after):
    was, now = {p["number"]: p for p in before}, {p["number"]: p for p in after}
    for n in sorted(set(was) - set(now)):
        yield f"PR #{n}: closed"
    for n in sorted(set(now) - set(was)):
        yield f"PR #{n}: opened by {now[n]['author']}, joins the queue - {now[n]['title']}"


def interrupted():
    """The session a terminated watcher left running, {"id", "started", "resumes"}, or None."""
    try:
        return json.loads(SESSION.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def run_session(dry_run, poller, interval, resume=None):
    """Run a session to its end - a new one, or `resume` (an interrupted one) continued - and forget it once it has
    reported. One that ends without a report (the watcher killed, the CLI crashed) stays in SESSION to be resumed."""
    claude = shutil.which("claude")
    if resume:
        sid, prompt, how = resume["id"], RESUME, ["--resume", resume["id"]]
    else:
        sid = str(uuid.uuid4())
        prompt, how = PROMPT, ["--session-id", sid]
    cmd = [claude or "claude", "-p", prompt, *how, "--permission-mode", "auto", "--output-format", "stream-json", "--verbose"]
    if dry_run:
        print(f"{stamp()} would run: claude -p <prompt> {' '.join(how)} --permission-mode auto --output-format stream-json --verbose", flush=True)
        return
    if not claude:
        sys.exit("pr-watch: claude is not on PATH")
    SESSION.write_text(json.dumps({"id": sid, "started": resume["started"] if resume else stamp(),
                                   "resumes": resume["resumes"] + 1 if resume else 0}), encoding="utf-8")
    log = STATE / (datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S") + ".jsonl")
    print(f"{stamp()} {'resuming' if resume else 'starting'} Claude Code session {sid}; its events go to {log}", flush=True)
    # `claude -p` otherwise stops background subagents 10 minutes after the session's last turn - the review agents,
    # killed mid-review with their slot leased and a break edit still in it.
    env = dict(os.environ, CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS="0")
    report = []

    def read(proc, out):
        for line in proc.stdout:
            out.write(line)
            out.flush()
            try:
                event = json.loads(line)
            except ValueError:
                print(f"{stamp()} {line.rstrip()}", flush=True)  # not an event: an error from the CLI itself
                continue
            lines, result = progress(event)
            for status in lines:
                print(f"{stamp()} {status}", flush=True)
            if result is not None:
                report.append(result)

    slots, prs = pool(), poller.prs
    with open(log, "a", encoding="utf-8") as out:
        proc = subprocess.Popen(cmd, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                stdin=subprocess.DEVNULL, text=True, encoding="utf-8", errors="replace")
        reader = threading.Thread(target=read, args=(proc, out), daemon=True)
        reader.start()
        # What the watcher sees for itself, so progress shows even when the session says nothing until it ends.
        while proc.poll() is None:
            reader.join(interval)
            now = pool()
            for status in pool_changes(slots, now):
                print(f"{stamp()} {status}", flush=True)
            slots = now
            try:
                now = poller.poll()
            except (OSError, ValueError) as e:
                print(f"{stamp()} poll failed: {e}", flush=True)
                continue
            for status in pr_changes(prs, now):
                print(f"{stamp()} {status}", flush=True)
            prs = now
        reader.join()
        code = proc.returncode
    if report:
        SESSION.unlink(missing_ok=True)
        print(f"{stamp()} the session ended (exit {code}); its report:", flush=True)
        print(report[-1].rstrip(), flush=True)
    else:
        print(f"{stamp()} the session stopped without a report (exit {code}); it is resumed next", flush=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--interval", type=int, default=60, help="seconds between polls (default 60)")
    ap.add_argument("--once", action="store_true", help="stop after one session")
    ap.add_argument("--mark-seen", action="store_true", help="mark today's open pull requests seen and start nothing for them")
    ap.add_argument("--include-drafts", action="store_true", help="draft pull requests are work too")
    ap.add_argument("--dry-run", action="store_true", help="print the session command instead of running it")
    args = ap.parse_args()

    STATE.mkdir(parents=True, exist_ok=True)
    held = lock()  # noqa: F841 - held until exit
    poller = Poller()
    if args.mark_seen:
        save_seen(poller.poll(fresh=True))

    print(f"{stamp()} watching {REPO} every {args.interval}s", flush=True)
    while True:
        try:
            prs = [p for p in poller.poll() if args.include_drafts or not p["draft"]]
        except (OSError, ValueError) as e:
            print(f"{stamp()} poll failed: {e}", flush=True)
            time.sleep(args.interval)
            continue
        seen = load_seen()
        new = [p for p in prs if seen.get(str(p["number"])) != p["sha"]]
        pending = interrupted()
        if pending and pending["resumes"] >= MAX_RESUMES:
            print(f"{stamp()} session {pending['id']} would not resume after {MAX_RESUMES} tries; a new session adopts its slots", flush=True)
            SESSION.unlink(missing_ok=True)
            pending = None
            new = new or prs
        if pending or new:
            if pending:
                print(f"{stamp()} session {pending['id']}, started {pending['started']}, was stopped before it reported", flush=True)
            for p in new:
                print(f"{stamp()} #{p['number']} {p['author']}: {p['title']}", flush=True)
            run_session(args.dry_run, poller, args.interval, pending)
            after = poller.poll(fresh=True)
            save_seen(after)
            left = [p for p in after if args.include_drafts or not p["draft"]]
            if left:
                print(f"{stamp()} still open, not taken up again until new commits: " + ", ".join(f"#{p['number']}" for p in left), flush=True)
            if args.once:
                return
            print(f"{stamp()} waiting again", flush=True)
        time.sleep(args.interval)


if __name__ == "__main__":
    main()
