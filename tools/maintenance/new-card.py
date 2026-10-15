"""Everything there is to read before porting a card to the native engine, in one call, and its file to start from.

    python tools/maintenance/new-card.py "Mind Stone" [--no-write]

Prints the card's content JSON and the scenarios that use it, and writes
crates/cantrip-cards/src/cards/<card_name>.rs over the card's generated constant if the
file does not exist yet. Then the loop is: write the behavior, and

    cargo run --profile cards -p cantrip-script --bin corpus -- --card "Mind Stone"

which runs only that card's scenarios and prints the full report of any that fail (docs/adding-cards.md).
"""

import json
import os
import re
import sys

import subprocess

# Oracle text carries characters such as U+2212 that the Windows console code page cannot print.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")


def _root():
    """The repository the command runs in, so a worktree's run writes into that worktree whichever copy of the script
    was named; else the script's own repository."""
    try:
        out = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
        if out:
            return os.path.normpath(out)
    except (OSError, subprocess.CalledProcessError):
        pass
    return os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))


ROOT = _root()
CONTENT = os.path.join(ROOT, "content")
SCRIPTS = os.path.join(ROOT, "scenarios")
CARDS = os.path.join(ROOT, "crates", "cantrip-cards", "src", "cards")


def ident(name):
    """The generated oracle constant (crates/cantrip-oracle/build.rs)."""
    s = re.sub(r"[^A-Za-z0-9]+", "_", name.replace("'", "")).strip("_").upper()
    return s if not s[0].isdigit() else "_" + s


def file_stem(name):
    return re.sub(r"[^a-z0-9]+", "_", name.replace("'", "").lower()).strip("_")



def find_card(name):
    for dataset in sorted(os.listdir(CONTENT)):
        folder = os.path.join(CONTENT, dataset, "cards")
        if not os.path.isdir(folder):
            continue
        for file in os.listdir(folder):
            if not file.endswith(".json"):
                continue
            path = os.path.join(folder, file)
            text = open(path, encoding="utf-8-sig").read()
            body = "\n".join(line for line in text.split("\n") if not line.strip().startswith("//"))
            try:
                card = json.loads(body)
            except ValueError:
                continue
            if card.get("name", "").lower() == name.lower():
                return dataset, path, text, card
    return None


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    found = find_card(args[0])
    if found is None:
        sys.exit(f"no card named {args[0]!r} in content/")
    dataset, path, text, card = found
    name = card["name"]

    print(f"=== {name}  ({os.path.relpath(path, ROOT)})  oracle::{ident(name)}")
    print(text.strip())

    print("\n=== Scenarios naming it")
    pattern = re.compile(r"^\s*(?:\d+\s+)?" + re.escape(name) + r"(?:\s+x\d+)?\s*$", re.M | re.I)
    for file in sorted(os.listdir(SCRIPTS)):
        if file.endswith(".cantrip") and pattern.search(open(os.path.join(SCRIPTS, file), encoding="utf-8-sig").read()):
            print(f"  scenarios/{file}")

    target = os.path.join(CARDS, file_stem(name) + ".rs")
    if os.path.exists(target):
        print(f"\n=== {os.path.relpath(target, ROOT)} exists:")
        print(open(target, encoding="utf-8").read().strip())
    elif "--no-write" not in sys.argv:
        with open(target, "w", encoding="utf-8", newline="\n") as f:
            f.write(
                "use cantrip_api::*;\nuse cantrip_oracle as oracle;\n\n"
                f"pub static CARD: CardDef = CardDef {{ ..oracle::{ident(name)} }};\n"
            )
        print(f"\n=== wrote {os.path.relpath(target, ROOT)} (characteristics only; add the behavior)")


if __name__ == "__main__":
    main()
