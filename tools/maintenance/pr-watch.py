"""Wait for pull requests on cantrip-rs/cantrip-rs, and clear the queue with a fresh Claude Code session each time.

    python tools/maintenance/pr-watch.py [--interval 60] [--once] [--mark-seen] [--include-drafts] [--dry-run]

tools\\pr-watch.cmd runs it. Every --interval seconds it asks GitHub's REST API for the open pull requests (`gh api`), with
the last answer's ETag: an unchanged list comes back 304 Not Modified, which costs nothing against the rate limit. A pull
request is new work when its head commit is one no session has been started for (a new PR, or new commits pushed to
one). New work starts `claude -p` in the repository root - a new session, so a new context each time - whose prompt has
it clear the whole queue through the `review-card-pr` skill, two review agents at a time, and stop. When it exits,
every open pull request's head is marked seen and the wait begins again; a PR the session left open (one waiting on a
human's decision) is not taken up again until it gets new commits.

Each session's output is written to .claude/pr-watch/<time>.log (gitignored), and what has been seen to
.claude/pr-watch/seen.json. --mark-seen marks today's open pull requests seen without starting a session, so only
later work wakes it; --once stops after one session; --dry-run prints the command instead of running it. Only one
watcher runs at a time (.claude/pr-watch/lock).
"""

import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO = "cantrip-rs/cantrip-rs"
API = f"repos/{REPO}/pulls?state=open&per_page=100"
ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / ".claude" / "pr-watch"
SEEN = STATE / "seen.json"

PROMPT = f"""Clear the pull-request queue of {REPO}.

List the open pull requests (`gh pr list --repo {REPO} --state open`) and land each one through the `review-card-pr`
skill and `add-cards/coordinator.md`, oldest first, with at most two review agents working at a time: a lease and a
cherry-pick, an Opus review agent, a Sonnet fix agent for findings that change code, the reviewer's confirmation, then
the landing (the test run it names, fetch and rebase, push, `card-issues.py log N landed` where the card has an issue,
`gh pr comment` with the hashes and what the review found, `gh pr close`, release the lease). A pull request that adds
no card still gets a review agent, against CLAUDE.md. List the queue again after each landing: new pull requests that
arrive while you work are part of it.

A pull request that needs a decision only the user can make (a rules exception, a cherry-pick conflict, a finding you
cannot settle) is left open and unlanded, its lease released with --discard, and named in your report; carry on with
the rest.

Do not start any wait, poll or sleep loop for new pull requests: something else does that. Do not end your turn while a
subagent or a test run you started is still going - wait for its notification. When no open pull request is left that
you can land, finish with a short report per pull request: what landed and its hashes, what the reviews found and
changed, and anything left for the user."""


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


def run_session(dry_run):
    claude = shutil.which("claude")
    cmd = [claude or "claude", "-p", PROMPT, "--permission-mode", "auto"]
    if dry_run:
        print(f"{stamp()} would run: claude -p <prompt> --permission-mode auto", flush=True)
        return
    if not claude:
        sys.exit("pr-watch: claude is not on PATH")
    log = STATE / (datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S") + ".log")
    print(f"{stamp()} starting a Claude Code session; its output goes to {log}", flush=True)
    # `claude -p` otherwise stops background subagents 10 minutes after the session's last turn - the review agents,
    # killed mid-review with their slot leased and a break edit still in it.
    env = dict(os.environ, CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS="0")
    with open(log, "w", encoding="utf-8") as out:
        code = subprocess.run(cmd, cwd=ROOT, env=env, stdout=out, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL).returncode
    print(f"{stamp()} the session ended (exit {code}); its report:", flush=True)
    print(log.read_text(encoding="utf-8", errors="replace").rstrip()[-4000:], flush=True)


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
        if new:
            for p in new:
                print(f"{stamp()} #{p['number']} {p['author']}: {p['title']}", flush=True)
            run_session(args.dry_run)
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
