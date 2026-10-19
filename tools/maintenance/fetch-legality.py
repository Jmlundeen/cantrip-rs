"""Each card's format legalities and color identity, from Scryfall's bulk data, into content/mtg/legality.json.

    python tools/maintenance/fetch-legality.py

Reads https://api.scryfall.com/bulk-data for the Oracle Cards file (one card object for each Oracle id), streams it,
and keeps, for every card that has a file under content/mtg/cards, its color identity (WUBRG order, empty for
colorless) and its legality in each format the deck editor builds for: vintage, legacy, modern, pioneer, standard,
commander and penny (Penny Dreadful). The file is what the bot's site serves the deck editor its catalog from
(dotnet/src/Cantrip.Discord/DeckSite.cs); run this again as formats move - a ban, a rotation, a new Penny season.

A card is matched by its name, or a double-faced or split card by its front face's name, which is what a card file
names it by. Tokens, emblems and art series are not cards; a playtest card, a digital Alchemy rebalancing (A-) and
an un-set card never stand in for the paper card of the same name. Prints the content names Scryfall had none for.
"""

import gzip
import io
import json
import os
import re
import subprocess
import sys
import urllib.request

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

FORMATS = ["vintage", "legacy", "modern", "pioneer", "standard", "commander", "penny"]
WUBRG = "WUBRG"
HEADERS = {"User-Agent": "cantrip-maintenance/1.0", "Accept": "application/json"}

# Not cards a deck is built from.
SKIPPED_LAYOUTS = {"token", "double_faced_token", "emblem", "art_series"}

# Printed or not, never the card a deck means by the name: worst first, so a paper card always wins.
LESSER_SET_TYPES = {"funny", "playtest", "memorabilia", "alchemy"}


def _root():
    """The repository the command runs in, so a worktree's run writes into that worktree; else the script's own."""
    try:
        out = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
        if out:
            return os.path.normpath(out)
    except (OSError, subprocess.CalledProcessError):
        pass
    return os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))


ROOT = _root()
DATASET = os.path.join(ROOT, "content", "mtg")


def content_names():
    """Every card name a card file defines: its first "name", the front face's."""
    names = set()
    cards = os.path.join(DATASET, "cards")
    for entry in sorted(os.listdir(cards)):
        if not entry.endswith(".json"):
            continue
        # Card files may carry // comments, so read the name with a regex rather than a JSON parser.
        text = open(os.path.join(cards, entry), encoding="utf-8").read()
        match = re.search(r'"name"\s*:\s*("(?:[^"\\]|\\.)*")', text)
        if match:
            names.add(json.loads(match.group(1)))
    return names


def fetch(url):
    request = urllib.request.Request(url, headers=HEADERS)
    return urllib.request.urlopen(request, timeout=120)


def oracle_cards():
    """The Oracle Cards bulk file's date, and its card objects one at a time."""
    with fetch("https://api.scryfall.com/bulk-data") as response:
        listing = json.load(response)
    entry = next(item for item in listing["data"] if item["type"] == "oracle_cards")
    url = entry.get("jsonl_download_uri") or entry["download_uri"]
    print(f"Reading {url}", file=sys.stderr)

    def cards():
        with fetch(url) as response:
            stream = gzip.GzipFile(fileobj=response) if url.endswith(".gz") else response
            if url.endswith(".jsonl") or url.endswith(".jsonl.gz"):
                for line in io.TextIOWrapper(stream, encoding="utf-8"):
                    line = line.strip()
                    if line:
                        yield json.loads(line)
            else:
                yield from json.load(stream)

    return entry["updated_at"], cards()


def rank(card):
    """How much a card object is the paper card of its name: lower is better."""
    worse = 0
    if card.get("set_type") in LESSER_SET_TYPES:
        worse += 2
    if "paper" not in card.get("games", ["paper"]):
        worse += 1
    return worse


def main():
    wanted = content_names()
    updated, cards = oracle_cards()
    found = {}
    for card in cards:
        if card.get("layout") in SKIPPED_LAYOUTS:
            continue
        name = card.get("name", "")
        if name.startswith("A-"):
            continue
        front = name.split(" // ")[0]
        key = name if name in wanted else front if front in wanted else None
        if key is None:
            continue
        if key in found and rank(found[key]) <= rank(card):
            continue
        found[key] = card

    out = {}
    for name in sorted(found):
        card = found[name]
        identity = "".join(color for color in WUBRG if color in card.get("color_identity", []))
        legalities = card.get("legalities", {})
        out[name] = {
            "identity": identity,
            "legalities": {format_: legalities.get(format_, "not_legal") for format_ in FORMATS},
        }

    path = os.path.join(DATASET, "legality.json")
    with open(path, "w", encoding="utf-8", newline="\n") as file:
        json.dump({"updated": updated, "cards": out}, file, ensure_ascii=False, indent=2, sort_keys=True)
        file.write("\n")

    missing = sorted(wanted - set(found))
    print(f"{len(found)} of {len(wanted)} cards written to {os.path.relpath(path, ROOT)}.")
    for name in missing:
        print(f"No Scryfall card: {name}")


if __name__ == "__main__":
    main()
