"""Which card faces and tokens have no art, for every card the engine has (a card file under content/*/cards/).

    python tools/maintenance/missing-art.py            every face name of every card (front, back face, split halves)
                                                        with no picture, one a line and nothing else
    python tools/maintenance/missing-art.py --tokens   the same for the tokens those cards make

Either narrows to the cards of one list: --list <Cube> (docs/cube-lists/<Cube>.txt) or --file <path> (any list of
names, what card-list.py writes: a decklist, a set).

Art is matched the way the client does it (CardArt.Slug): the slug of a face or token name against the file names
under content/*/images/, a predefined token's without its " Token" (CR 111.4: the Treasure token is "Treasure Token",
drawn with treasure.jpg). A card counts once it has a card file. A card's tokens are read from its Rust file
(crates/cantrip-cards/src/cards/<card>.rs): the token definitions it declares (a `CardDef` built on `CardDef::BLANK`)
and the shared ones it names from `cantrip_mechanics::tokens` (`tokens::TREASURE`, `tokens::create_treasure`).
"""

import collections
import json
import os
import re
import sys
import unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CARD_CODE = os.path.join(ROOT, "crates", "cantrip-cards", "src", "cards")
SHARED_TOKENS = os.path.join(ROOT, "crates", "cantrip-mechanics", "src", "tokens.rs")
# A `CardDef` static built on BLANK, and its name: how a token is declared in Rust.
TOKEN_DEF = re.compile(r'static\s+([A-Z][A-Z0-9_]*)\s*:\s*CardDef\s*=\s*CardDef\s*\{(.*?)\.\.CardDef::BLANK', re.S)


def slug(name):
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c)).lower()
    folded = folded.replace("'", "").replace("’", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", folded)).strip("-")


def code_stem(name):
    """The card's Rust file name (new-card.py's rule)."""
    return slug(name).replace("-", "_")


def read(path):
    with open(path, encoding="utf-8-sig") as handle:
        return handle.read()


def listed_names(only, path=None):
    if path:
        if not os.path.isfile(path):
            sys.exit("no file " + path)
        names = collections.OrderedDict()
        for line in read(path).splitlines():
            name = re.sub(r"^\d+x?\s+", "", line.strip())
            if name and not name.startswith("#"):
                names[name] = True
        return list(names)
    folder = os.path.join(ROOT, "docs", "cube-lists")
    if not os.path.isfile(os.path.join(folder, only + ".txt")):
        sys.exit("no cube list " + os.path.join(folder, only + ".txt"))
    names = collections.OrderedDict()
    for line in read(os.path.join(folder, only + ".txt")).splitlines():
        name = re.sub(r"^\d+x?\s+", "", line.strip())
        if name and not name.startswith("#"):
            names[name] = True
    return list(names)


def art_slugs():
    found = set()
    content = os.path.join(ROOT, "content")
    for dataset in os.listdir(content):
        images = os.path.join(content, dataset, "images")
        if os.path.isdir(images):
            for entry in os.listdir(images):
                stem, extension = os.path.splitext(entry)
                if extension.lower() in (".png", ".jpg", ".jpeg", ".webp"):
                    found.add(stem.lower())
    return found


def card_files():
    by_name = {}
    content = os.path.join(ROOT, "content")
    for dataset in sorted(os.listdir(content)):
        folder = os.path.join(content, dataset, "cards")
        if os.path.isdir(folder):
            for entry in sorted(os.listdir(folder)):
                if entry.endswith(".json"):
                    card = json.loads(read(os.path.join(folder, entry)))
                    by_name[slug(card["name"])] = card
    return by_name


def faces(card):
    # A split card is drawn as its halves, each with its own art (CardFace.ArtNames). An adventure, omen or prepare
    # face has no art of its own: it is drawn with the card's.
    halves = card.get("halves") or []
    names = [half["name"] for half in halves] if halves else [card["name"]]
    if isinstance(card.get("backFace"), dict) and card["backFace"].get("name"):
        names.append(card["backFace"]["name"])
    return names


def token_names(text):
    """The tokens a Rust file declares, by static: every `CardDef` built on BLANK but an emblem, which has no art."""
    names = {}
    for static, body in TOKEN_DEF.findall(text):
        name = re.search(r'name:\s*"([^"]+)"', body)
        if name and not name.group(1).lower().endswith(" emblem"):
            names[static] = name.group(1)
    return names


def tokens_of(card, shared):
    path = os.path.join(CARD_CODE, code_stem(card["name"]) + ".rs")
    if not os.path.isfile(path):
        return []
    text = read(path)
    names = set(token_names(text).values())
    for static in re.findall(r"tokens::([A-Z][A-Z0-9_]*)\b", text):
        if static in shared:
            names.add(shared[static])
    for helper in re.findall(r"tokens::create_([a-z_]+)\b", text):
        if helper.upper() in shared:
            names.add(shared[helper.upper()])
    if re.search(r"tokens::incubate\b", text) and "INCUBATOR" in shared:
        names.add(shared["INCUBATOR"])
    return sorted(names)


def main():
    args = sys.argv[1:]
    only = args[args.index("--list") + 1] if "--list" in args else None
    path = args[args.index("--file") + 1] if "--file" in args else None
    want_tokens = "--tokens" in args
    art = art_slugs()
    cards = card_files()
    shared = token_names(read(SHARED_TOKENS))
    missing = []
    if only or path:
        chosen = [cards[slug(name)] for name in listed_names(only, path) if slug(name) in cards]
    else:
        chosen = list(cards.values())
    for card in chosen:
        for face in (tokens_of(card, shared) if want_tokens else faces(card)):
            drawn = face[: -len(" Token")] if want_tokens and face.endswith(" Token") else face
            if slug(drawn) not in art and face not in missing:
                missing.append(face)
    missing.sort(key=lambda n: (slug(n), n))
    for name in missing:
        print(name)


if __name__ == "__main__":
    main()
