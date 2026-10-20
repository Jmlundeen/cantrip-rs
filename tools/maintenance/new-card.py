"""Everything there is to read before porting a card to the native engine, in one call, and its file to start from.

    python tools/maintenance/new-card.py "Mind Stone" [--no-write]

Prints the card's content JSON, the scenarios that use it and, for each of its abilities, the implemented cards that
already do it (ability-index.py), and writes
crates/cantrip-cards/src/cards/<card_name>.rs over the card's generated constant if the
file does not exist yet. Then the loop is: write the behavior, and

    cargo run --profile cards -p cantrip-script --bin corpus -- --card "Mind Stone"

which runs only that card's scenarios and prints the full report of any that fail (docs/adding-cards.md).
"""

import importlib.util
import json
import os
import re
import sys

import subprocess

import cr
import scryfall

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
GAME_RS = os.path.join(ROOT, "crates", "cantrip-api", "src", "game.rs")


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


def _abilities(index, text, names):
    """A face's abilities as compared with Scryfall's: one kind of quote and dash, case aside."""
    text = text.replace("’", "'").replace("‘", "'").replace("“", '"').replace("”", '"').replace("−", "-")
    text = re.sub(r"(?<=\S) - (?=\S)|\s*-\s*$", lambda m: " — " if m.group(0).strip() else " —", text, flags=re.M)
    return sorted(index.template(part, "exact") for ab in index.split_abilities(text, names) for _, part in ab.parts)


def scryfall_report(card, index):
    """Whether each face's Oracle text in content/ is Scryfall's current text, and the card's rulings."""
    found = scryfall.card(card["name"])
    if found is None:
        return f"  not found on Scryfall (or Scryfall unreachable): check the Oracle text and rulings by hand"
    theirs = {f["name"].lower(): f["oracle_text"] for f in found["faces"]}
    if len(found["faces"]) == 1:
        theirs[found["name"].lower()] = found["faces"][0]["oracle_text"]
    lines = []
    names = [n for n, _ in index.card_texts(card)] + [card["name"]]
    for face, mine in index.card_texts(card):
        current = theirs.get((face or card["name"]).lower())
        if current is None:
            if "//" not in card["name"]:  # a split card's halves are faces of the whole card's name
                lines.append(f"  {face}: no face of that name on Scryfall")
        # Compared ability by ability: the card's name and "this creature" are one (Oracle updates move between them),
        # reminder text is no rule, and "Flying, lifelink" is "Flying" and "Lifelink" on lines of their own.
        elif _abilities(index, mine, names) != _abilities(index, current, names):
            lines.append(f"  {face}: DIFFERS from Scryfall's current Oracle text - fix the content JSON first:")
            lines.append("    " + current.replace("\n", "\n    "))
    if not lines:
        lines.append("  Oracle text matches Scryfall's")
    rulings = found["rulings"]
    lines.append(f"  {len(rulings)} ruling{'' if len(rulings) == 1 else 's'}{':' if rulings else ''}")
    for r in rulings:
        lines.append(f"  - {r['date']}  {r['text']}")
    return "\n".join(lines)


# Keyword actions the core does in its own procedures (casting, activating, searching ...): their number is enough.
BASIC_ACTIONS = {"activate", "attach", "cast", "counter", "create", "destroy", "discard", "double", "exchange", "exile",
                 "play", "reveal", "sacrifice", "search", "shuffle", "tap and untap", "transform", "fight", "mill"}


def _has_verb(stem):
    """Whether cantrip-api's Game has a verb of that name ("scry", "sacrifice")."""
    try:
        return re.search(r"\bfn " + re.escape(stem) + r"\b", open(GAME_RS, encoding="utf-8").read()) is not None
    except OSError:
        return False


def cr_report(card, abilities, index):
    """The CR rule of each keyword and keyword action on the card: the whole rule when nothing in the engine does it yet
    (the rule is the specification), its number when the core or cantrip-mechanics already does."""
    core = index.core_keywords()
    lines, seen = [], set()
    for ab in abilities:
        if ab.keyword is None:
            continue
        found = cr.section(ab.keyword)
        if found is None or found[0] in seen:
            continue
        seen.add(found[0])
        number, title = found
        if ab.keyword in core:
            lines.append(f"  {number} {title}: core keyword")
        elif index.mechanics_for(ab.keyword):
            lines.append(f"  {number} {title}: {index.mechanics_for(ab.keyword)}")
        else:
            lines.append(f"  {number} {title}: no module yet - its rule:\n    " + cr.rule(number).replace("\n", "\n    "))
    names = [n for n, _ in index.card_texts(card)] + [card["name"]]
    text = index.clean("\n".join(t for _, t in index.card_texts(card)), names)  # "Enduring Courage" endures nothing
    for number, title in cr.terms_in(text):
        if number in seen:
            continue
        stem = index.file_stem(title)
        home = f"g.{stem}" if _has_verb(stem) else index.mechanics_for(title)
        if title.lower() in BASIC_ACTIONS:
            lines.append(f"  {number} {title}" + (f": {home}" if home else ""))
        elif home:
            lines.append(f"  {number} {title}: {home}")
        else:
            lines.append(f"  {number} {title}: no verb or module of that name - its rule:\n    " + cr.rule(number).replace("\n", "\n    "))
    return "\n".join(lines) or "  none"


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

    spec = importlib.util.spec_from_file_location("ability_index", os.path.join(os.path.dirname(__file__), "ability-index.py"))
    index = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(index)
    abilities = index.card_abilities(card)

    print("\n=== Scryfall: current Oracle text and official rulings (tools/maintenance/scryfall.py)")
    print(scryfall_report(card, index))

    print("\n=== Comprehensive Rules: the card's keywords and keyword actions (tools/maintenance/cr.py <number or name>)")
    print(cr_report(card, abilities, index))

    print("\n=== Prior art: the implemented cards that already do each ability (tools/maintenance/ability-index.py)")
    print(index.report(index.Index(exclude=[name]), abilities) or "  (no rules text)")

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
