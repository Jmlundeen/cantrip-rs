"""Everything there is to read before porting a card to the native engine, in one call, and its file to start from.

    python tools/maintenance/new-card.py "Mind Stone" [--no-write]

Writes the card's content JSON from Scryfall when content/ has none (a single face, a double-faced card, an adventure;
any other layout is written by hand), then prints the card's content JSON, the scenarios that use it and, for each of its abilities, the implemented cards that
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
import unicodedata

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


# Scryfall layouts a content file says as one face, a back face or an adventure; the rest are written by hand.
SINGLE = {"normal", "leveler", "class", "saga", "case", "mutate"}
COLOR_NAMES = {"W": "White", "U": "Blue", "B": "Black", "R": "Red", "G": "Green"}


def _core_keywords():
    """{"first strike": "FirstStrike", ...}: the Oracle keywords that are the core's (crates/cantrip-oracle/build.rs)."""
    src = open(os.path.join(ROOT, "crates", "cantrip-oracle", "build.rs"), encoding="utf-8").read()
    names = re.findall(r'"([^"]+)"', re.search(r"const KEYWORDS[^=]*=\s*\[(.*?)\];", src, re.S).group(1))
    words = re.findall(r'"([^"]+)"', re.search(r"const KEYWORD_WORDS[^=]*=\s*\[(.*?)\];", src, re.S).group(1))
    by_key = {n.lower(): n for n in names}
    return {w: by_key[w.replace(" ", "")] for w in words if w.replace(" ", "") in by_key}


def _stat(value):
    return int(value) if re.fullmatch(r"-?\d+", value or "") else value


def _face(face, names, index, core):
    """One face as a content file has it: its characteristics, and the core keywords its Oracle text has."""
    out = {"name": face["name"]}
    if face.get("mana_cost"):
        out["manaCost"] = face["mana_cost"]
    if face.get("color_indicator"):
        out["colorIndicator"] = [COLOR_NAMES[c] for c in face["color_indicator"]]
    out["type"] = face["type_line"]
    for key in ("power", "toughness", "loyalty", "defense"):
        if face.get(key):
            out[key] = _stat(face[key])
    out["oracleText"] = face["oracle_text"]
    words = [ab.keyword for ab in index.split_abilities(face["oracle_text"], names) if ab.keyword]
    keywords = [core[w] for w in dict.fromkeys(words) if w in core]
    if keywords:
        out["keywords"] = keywords
    for flag in ("convoke", "delve", "improvise"):
        if flag in words:
            out[flag] = True
    return out


def draft_content(name, index):
    """Writes content/mtg/cards/<slug>.json for a card new to content, from Scryfall's current characteristics; for a
    layout a file can't be written from directly (split, omen, prototype ...), prints what Scryfall has and exits."""
    found = scryfall.card(name)
    if found is None:
        sys.exit(f"no card named {name!r} in content/ or on Scryfall; spell it as printed")
    faces, layout = found["faces"], found.get("layout", "normal")
    names = [f["name"] for f in faces] + [found["name"]]
    core = _core_keywords()
    if layout in SINGLE and len(faces) == 1:
        card = _face(faces[0], names, index, core)
    elif layout in ("transform", "modal_dfc") and len(faces) == 2:
        card = _face(faces[0], names, index, core) | {"backFace": _face(faces[1], names, index, core)}
    elif layout == "adventure" and len(faces) == 2:
        card = _face(faces[0], names, index, core) | {"adventure": _face(faces[1], names, index, core)}
    else:
        print(f"{found['name']} is a {layout} card: write content/mtg/cards/<name>.json by hand (the add-cards skill, step 2),")
        print("then run new-card.py again. What Scryfall has:\n")
        for face in faces:
            print(f"{face['name']}  {face.get('mana_cost', '')}  {face.get('type_line', '')}\n{face['oracle_text']}\n")
        sys.exit(1)
    slug = re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", unicodedata.normalize("NFKD", card["name"])
                                     .encode("ascii", "ignore").decode().lower().replace("'", ""))).strip("-")
    path = os.path.join(CONTENT, "mtg", "cards", slug + ".json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(card, indent=2, ensure_ascii=False) + "\n")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    found = find_card(args[0])
    spec = importlib.util.spec_from_file_location("ability_index", os.path.join(os.path.dirname(__file__), "ability-index.py"))
    index = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(index)
    drafted = False
    if found is None:
        if "--no-write" in sys.argv:
            sys.exit(f"no card named {args[0]!r} in content/ (run without --no-write to write it from Scryfall)")
        draft_content(args[0], index)
        found = find_card(args[0])
        drafted = True
    dataset, path, text, card = found
    name = card["name"]

    print(f"=== {name}  ({os.path.relpath(path, ROOT)})  oracle::{ident(name)}")
    if drafted:
        print("    written just now from Scryfall: check it against the card (the core keywords besides Oracle's -")
        print("    CantBeBlocked, CantBeCountered ... in crates/cantrip-oracle/build.rs - are yours to add where the text says so)")
    print(text.strip())

    print("\n=== Scenarios naming it")
    pattern = re.compile(r"^\s*(?:\d+\s+)?" + re.escape(name) + r"(?:\s+x\d+)?\s*$", re.M | re.I)
    for file in sorted(os.listdir(SCRIPTS)):
        if file.endswith(".cantrip") and pattern.search(open(os.path.join(SCRIPTS, file), encoding="utf-8-sig").read()):
            print(f"  scenarios/{file}")

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
