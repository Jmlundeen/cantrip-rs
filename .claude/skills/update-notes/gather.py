"""Gathers what went into the public repository, jschibel/cantrip-rs, since the last update notes.

    python .claude/skills/update-notes/gather.py           prints the facts the notes are written from
    python .claude/skills/update-notes/gather.py --save SHA records SHA as the last commit reported

The public main is fetched into this repository's object store (no remote is added) and read with git. The last
commit reported is kept in last-reported-commit.txt beside this script, which is gitignored.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

PUBLIC = "https://github.com/jschibel/cantrip-rs.git"
STATE = Path(__file__).with_name("last-reported-commit.txt")
CARDS = "content/mtg/cards/"
MECHANICS = "crates/cantrip-mechanics/src/"
KNOWN_ISSUES = "docs/known-issues.md"


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True, encoding="utf-8").stdout


def card_names(rev: str, path: str) -> list[str]:
    card = json.loads(git("show", f"{rev}:{path}"))
    back = card.get("backFace", {}).get("name")
    return [f"{card['name']} // {back}" if back else card["name"]]


def main() -> None:
    if len(sys.argv) == 3 and sys.argv[1] == "--save":
        sha = git("rev-parse", "--verify", sys.argv[2] + "^{commit}").strip()
        STATE.write_text(sha + "\n", encoding="utf-8")
        print(f"last reported commit: {sha}")
        return

    if not STATE.exists():
        sys.exit(f"{STATE} is missing: write the sha of the last commit reported into it")
    last = STATE.read_text(encoding="utf-8").strip()
    git("fetch", "--quiet", PUBLIC, "main")
    head = git("rev-parse", "FETCH_HEAD").strip()
    print(f"FROM {last}\nTO   {head}")
    if head == last:
        print("\nNothing new since the last update.")
        return
    span = f"{last}..{head}"

    print("\n== NEW CARDS (card files added under content/mtg/cards) ==")
    for path in git("diff", "--diff-filter=A", "--name-only", last, head, "--", CARDS).split():
        for name in card_names(head, path):
            print(name)

    print("\n== FILES ADDED TO cantrip-mechanics ==")
    print(git("diff", "--diff-filter=A", "--name-only", last, head, "--", MECHANICS).strip() or "(none)")

    print("\n== KNOWN ISSUES CLOSED (entries deleted from docs/known-issues.md) ==")
    diff = git("diff", last, head, "--", KNOWN_ISSUES)
    removed = [l[1:] for l in diff.splitlines() if re.match(r"^-### KI-\d+", l)]
    added = {l[1:] for l in diff.splitlines() if re.match(r"^\+### KI-\d+", l)}
    print("\n".join(r for r in removed if r not in added) or "(none)")

    print("\n== COMMITS, oldest first ==")
    log = git("log", "--reverse", "--no-merges", "--format=--- %h %s%n%b", span)
    print("\n".join(l for l in log.splitlines() if not l.startswith("Co-Authored-By:")).strip())


if __name__ == "__main__":
    main()
