"""Which cube-list cards have no card file yet.

Reads every list in docs/cube-lists/ (any *.txt, so a new list needs no change here) and every card under
content/*/cards/, and prints the names on a list that no card file defines. A name matches a card either by
the card's "name" field or by the file-name slug the loader requires, both case- and punctuation-insensitive.

    python tools/maintenance/missing-cards.py           every missing name, with the lists it is on
    python tools/maintenance/missing-cards.py --pick    the first missing name alphabetically, and nothing else
    python tools/maintenance/missing-cards.py --pick 5  the first five, one per line, in the same order
    python tools/maintenance/missing-cards.py --count   how many are missing, of how many distinct names

Any of them takes --list <Cube> to look at that one list (docs/cube-lists/<Cube>.txt) instead of all of them, or
--file <path> to look at any list of names (what card-list.py writes, a decklist, a set's cards).

--pick is deterministic on purpose: the backlog is worked in one stable order, and the same name keeps coming
back until a card file defines it. Nothing is ever skipped for being hard, so nothing needs choosing between.
--pick N is for handing out cards to parallel agents: the next N names, still in that one order.
"""

import argparse
import json
import os
import re
import sys
import unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LISTS = os.path.join(ROOT, "docs", "cube-lists")
CONTENT = os.path.join(ROOT, "content")


def slug(name):
    """The card file name the loader requires, minus the extension."""
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    folded = folded.lower().replace("'", "").replace("’", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", folded)).strip("-")


def implemented():
    """Every name and slug that a card file under content/ defines."""
    known = set()
    for dataset in sorted(os.listdir(CONTENT)) if os.path.isdir(CONTENT) else []:
        cards = os.path.join(CONTENT, dataset, "cards")
        if not os.path.isdir(cards):
            continue
        for entry in sorted(os.listdir(cards)):
            if not entry.endswith(".json"):
                continue
            known.add(slug(entry[: -len(".json")]))
            # Card files carry // comments, so read the name with a regex rather than a JSON parser.
            text = open(os.path.join(cards, entry), encoding="utf-8").read()
            match = re.search(r'"name"\s*:\s*("(?:[^"\\]|\\.)*")', text)
            if match:
                known.add(slug(json.loads(match.group(1))))
    return known


def listed(only=None, path=None):
    """Every name on a cube list (or on the one list named, or in the one file given), in list order, mapped to the
    lists it is on."""
    names = {}
    if path:
        if not os.path.isfile(path):
            sys.exit("no file " + path)
        for line in open(path, encoding="utf-8-sig"):
            name = re.sub(r"^\d+x?\s+", "", line.strip())
            if name and not name.startswith("#"):
                names.setdefault(name, [os.path.basename(path)])
        return names
    if not os.path.isdir(LISTS):
        sys.exit("no cube lists at " + LISTS)
    if only and not os.path.isfile(os.path.join(LISTS, only + ".txt")):
        sys.exit("no cube list " + os.path.join(LISTS, only + ".txt"))
    for entry in sorted(os.listdir(LISTS)):
        if not entry.endswith(".txt") or (only and entry != only + ".txt"):
            continue
        for line in open(os.path.join(LISTS, entry), encoding="utf-8"):
            name = line.strip()
            if not name or name.startswith("#"):
                continue
            name = re.sub(r"^\d+x?\s+", "", name)  # "3 Card Name", as some exports write it
            lists = names.setdefault(name, [])
            if entry not in lists:  # a card the cube holds twice is still one name to build
                lists.append(entry)
    return names


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pick", type=int, nargs="?", const=1, metavar="N",
                        help="print the first N missing names alphabetically (default 1), and nothing else")
    parser.add_argument("--count", action="store_true", help="print how many are missing")
    parser.add_argument("--list", metavar="CUBE", help="only this list, docs/cube-lists/<CUBE>.txt")
    parser.add_argument("--file", metavar="PATH", help="only the names in this file, one a line")
    args = parser.parse_args()

    known = implemented()
    names = listed(args.list, args.file)

    # Alphabetical by the card's file-name slug, which is where a person would file it: punctuation and case are
    # already folded out, so '"Name Sticker" Goblin' sorts under N rather than ahead of everything on its quote.
    # One order everywhere — the first line of the bare listing is exactly what --pick prints.
    missing = sorted(
        ((name, lists) for name, lists in names.items() if slug(name) not in known),
        key=lambda pair: (slug(pair[0]), pair[0]),
    )

    if args.count:
        print("%d of %d listed names have no card file" % (len(missing), len(names)))
        return

    if args.pick is not None:
        if not missing:
            sys.exit("every listed card is implemented")
        # No preference and no randomness: whoever runs this next gets the same card until a file defines it.
        for name, _ in missing[: max(args.pick, 1)]:
            print(name)
        return

    for name, lists in missing:
        print("%-40s %s" % (name, " ".join(lists)))


if __name__ == "__main__":
    main()
