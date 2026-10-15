"""Any list of cards, as the card names the card files use, one a line: a decklist, a set, a Scryfall search, a text
file or names pasted in.

    python tools/maintenance/card-list.py <source> [--out FILE] [--main-only]

<source> is one of:

    path/to/file.txt         a decklist or a list of names (decks/rw-aggro.txt, an MTGO or Arena export, a cube list)
    -                        the same, read from standard input (a pasted list)
    set:<code or name>       every card of a set on Scryfall (set:mh3, "set:Modern Horizons 3")
    query:<scryfall search>  every card a Scryfall search finds ("query:o:earthbend")
    cube:<cube>              a CubeCobra cube's mainboard, read and not kept (cube:LSVCube, or its address); to keep a
                             cube as a list in docs/cube-lists, use cubecobra-sync.py instead

A decklist line may be "Name", "4 Name", "4x Name", "4 Name (SET) 123" or "SB: 2 Name"; # and // comments, blank lines
and section headers ("Deck", "Sideboard", "Commander", "Companion", "Maybeboard") are skipped. The sideboard, commander
and companion are part of the list unless --main-only is given; a maybeboard is never part of it.

Every name is checked against Scryfall and written as the card files spell it: the front face of a double-faced,
adventure, flip or meld card, "A // B" for a split card, and a Through the Omenpaths reprint as the original card's name
(cubecobra-sync.py's mapping, docs/cube-lists/printed-names.json). A name Scryfall does not know is reported on
standard error, and the exit status is 1, so a typo is never silently dropped; a name only a fuzzy match finds is
reported there too, with the card it was taken for. Each card is listed once, in the order
the source gives them. Without --out the names are printed.
"""

import argparse
import importlib.util
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

HERE = os.path.dirname(os.path.abspath(__file__))
HEADERS = {"User-Agent": "cantrip-card-list/1.0", "Accept": "application/json"}
SECTIONS = {"deck", "main", "mainboard", "sideboard", "commander", "companion", "maybeboard", "considering"}
EXTRA = {"sideboard", "commander", "companion"}
NOT_CARDS = {"token", "double_faced_token", "emblem", "art_series", "planar", "scheme", "vanguard"}


def cubecobra():
    """cubecobra-sync.py as a module: its CubeCobra fetch and Omenpaths mapping are used, not copied."""
    sys.dont_write_bytecode = True  # no __pycache__ left in tools/maintenance
    spec = importlib.util.spec_from_file_location("cubecobra_sync", os.path.join(HERE, "cubecobra-sync.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def scryfall(url, body=None):
    """One Scryfall request, at most ten a second as Scryfall asks."""
    time.sleep(0.1)
    data = json.dumps(body).encode("utf-8") if body is not None else None
    headers = dict(HEADERS, **({"Content-Type": "application/json"} if data else {}))
    request = urllib.request.Request(url, data=data, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return {"object": "error", "status": 404, "details": json.load(error).get("details", "")}
        sys.exit(f"Scryfall: {error}")
    except urllib.error.URLError as error:
        sys.exit(f"Scryfall: {error}")


def search(query):
    """Every card a Scryfall search finds, one printing each, in set-number order."""
    url = "https://api.scryfall.com/cards/search?" + urllib.parse.urlencode({"q": query, "unique": "cards", "order": "set"})
    cards = []
    while url:
        page = scryfall(url)
        if page.get("object") == "error":
            sys.exit(f"Scryfall finds nothing for {query!r}: {page.get('details', '')}")
        cards.extend(page["data"])
        url = page.get("next_page") if page.get("has_more") else None
    return cards


def set_code(text):
    """A set's code, from its code or its name, case-insensitive."""
    sets = scryfall("https://api.scryfall.com/sets")["data"]
    wanted = text.strip().lower()
    for s in sets:
        if s["code"] == wanted or s["name"].lower() == wanted:
            return s["code"]
    close = [f"{s['name']} ({s['code']})" for s in sets if wanted in s["name"].lower()]
    sys.exit(f"no set {text!r} on Scryfall" + (": did you mean " + ", ".join(close[:8]) + "?" if close else ""))


def read_decklist(text, main_only):
    """The names a decklist or name list holds, in order, each once."""
    names, section = [], "deck"
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith("//"):
            continue
        header = line.rstrip(":").strip().lower()
        if header in SECTIONS:
            section = header
            continue
        sideboard = line.upper().startswith("SB:")
        if sideboard:
            line = line[3:].strip()
        if section in ("maybeboard", "considering") or (main_only and (sideboard or section in EXTRA)):
            continue
        line = re.sub(r"^\d+x?\s+", "", line)  # a count
        line = re.sub(r"\s+\*[A-Z]+\*$", "", line)  # *CMDR*, *F* markers
        line = re.sub(r"\s+\([A-Za-z0-9]{2,6}\)(\s+\S+)?$", "", line)  # (SET) and a collector number
        if line and line not in names:
            names.append(line)
    return names


def card_name(card, omenpaths):
    """The name the card files use for a Scryfall card."""
    if card.get("set") in omenpaths.RENAMED_SETS:
        return omenpaths.original_name(card)
    if " // " in card["name"] and card.get("layout") != "split":
        return card["name"].split(" // ")[0]
    return card["name"]


def resolve(names):
    """The Scryfall card for each name (exact first, then a fuzzy match), and the names it does not know."""
    found, unknown = {}, []
    for start in range(0, len(names), 75):
        batch = names[start:start + 75]
        answer = scryfall("https://api.scryfall.com/cards/collection", {"identifiers": [{"name": n} for n in batch]})
        by_name = {}
        for card in answer.get("data", []):
            by_name[card["name"].lower()] = card
            for face in card.get("card_faces", []):
                by_name.setdefault(face["name"].lower(), card)
        for name in batch:
            card = by_name.get(name.lower())
            if card is None:
                fuzzy = scryfall("https://api.scryfall.com/cards/named?" + urllib.parse.urlencode({"fuzzy": name}))
                card = None if fuzzy.get("object") == "error" else fuzzy
                if card is not None:
                    print(f"matched {name!r} as {card['name']!r}", file=sys.stderr)
            if card is None:
                unknown.append(name)
            else:
                found[name] = card
    return found, unknown


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("source", help="a file, -, set:<code or name>, query:<search> or cube:<cube>")
    parser.add_argument("--out", metavar="FILE", help="write the names to FILE instead of printing them")
    parser.add_argument("--main-only", action="store_true", help="leave out a decklist's sideboard, commander and companion")
    args = parser.parse_args()

    omenpaths = cubecobra()
    source, unknown = args.source, []
    if source.startswith("set:"):
        cards = search(f"e:{set_code(source[4:])} -is:token")
    elif source.startswith("query:"):
        cards = search(source[6:])
    elif source.startswith("cube:"):
        cube = omenpaths.fetch_cube(omenpaths.cube_id(source[5:]))
        cards = [c["details"] for c in cube["cards"]["mainboard"]]
    else:
        if source == "-":
            text = sys.stdin.read()
        elif os.path.isfile(source):
            with open(source, encoding="utf-8-sig") as f:
                text = f.read()
        else:
            sys.exit(f"no file {source!r} (a set, search or cube is set:, query: or cube:)")
        found, unknown = resolve(read_decklist(text, args.main_only))
        cards = list(found.values())

    names = []
    for card in cards:
        if card.get("layout") in NOT_CARDS:
            continue
        name = card_name(card, omenpaths)
        if name not in names:
            names.append(name)

    output = "".join(name + "\n" for name in names)
    if args.out:
        with open(args.out, "w", encoding="utf-8", newline="\n") as f:
            f.write(output)
        print(f"{len(names)} cards to {args.out}")
    else:
        sys.stdout.write(output)
    for name in unknown:
        print(f"unknown card: {name}", file=sys.stderr)
    sys.exit(1 if unknown else 0)


if __name__ == "__main__":
    main()
