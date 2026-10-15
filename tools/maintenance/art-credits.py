"""Fills in who illustrated each card picture, from its original printing on Scryfall.

For every picture under content/<dataset>/images/ named by slug, and not yet credited in that dataset's
artists.json, looks the card up on Scryfall and writes down the artist of its earliest paper printing that is
not a promo (falling back to the earliest printing of any kind): the art kept here is taken to be the original
printing's. A double-faced card's face is credited with that face's artist. Credits already written are kept,
so a picture swapped for another printing's is corrected by hand in artists.json and stays corrected.

    python tools/maintenance/art-credits.py            fill in what is missing, for every dataset
    python tools/maintenance/art-credits.py --dry-run  print what would be written, write nothing

The art service serves the datasets' credits merged, at /art/artists.json (docs/web-client.md).
"""

import argparse
import json
import os
import re
import sys
import time
import unicodedata
import urllib.error
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CONTENT = os.path.join(ROOT, "content")
SLUG = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")
PICTURES = (".png", ".jpg", ".jpeg", ".webp")


def slug(name):
    """The art file convention, as CardArt.Slug has it: accents folded, lowercase, apostrophes dropped."""
    out, separator = [], False
    for ch in unicodedata.normalize("NFD", name):
        if unicodedata.category(ch) == "Mn":
            continue
        c = ch.lower()
        if c in "'’":
            continue
        if c.isascii() and c.isalnum():
            if separator and out:
                out.append("-")
            out.append(c)
            separator = False
        else:
            separator = True
    return "".join(out)


def card_names(dataset):
    """Every name a card file of the dataset gives - its own, its faces' - by slug."""
    names = {}

    def walk(node):
        if isinstance(node, dict):
            if isinstance(node.get("name"), str):
                names.setdefault(slug(node["name"]), node["name"])
            for value in node.values():
                walk(value)
        elif isinstance(node, list):
            for value in node:
                walk(value)

    cards = os.path.join(dataset, "cards")
    for file in sorted(os.listdir(cards)) if os.path.isdir(cards) else []:
        if not file.endswith(".json"):
            continue
        with open(os.path.join(cards, file), encoding="utf-8-sig") as f:
            text = "\n".join(line for line in f.read().splitlines() if not line.lstrip().startswith("//"))
        try:
            walk(json.loads(text))
        except json.JSONDecodeError:
            pass
    return names


def fetch(url):
    """An answer from Scryfall, or None when it has none; waits and asks again when asked to slow down."""
    request = urllib.request.Request(url, headers={"User-Agent": "CantripArtCredits/1.0", "Accept": "application/json"})
    for attempt in range(8):
        time.sleep(0.5)  # Well under Scryfall's ten a second: its search is throttled harder than that.
        try:
            with urllib.request.urlopen(request) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            if error.code != 429:
                return None
            time.sleep(min(60, 2 ** (attempt + 1)))
    raise RuntimeError(f"Scryfall kept refusing {url}")


def scryfall(path, **query):
    return fetch(f"https://api.scryfall.com/{path}?{urllib.parse.urlencode(query)}")


def printings(query):
    """Every printing a search finds, oldest first, following its pages."""
    page = scryfall("cards/search", q=query, unique="prints", order="released", dir="asc")
    found = []
    while page is not None:
        found.extend(page["data"])
        page = fetch(page["next_page"]) if page.get("has_more") else None
    return found


def faces(card):
    return card.get("card_faces") or [card]


def original_artist(want, prints):
    """The artist of the earliest paper, non-promo printing with a face whose slug is `want`, from `prints`."""
    prints = [p for p in prints if any(slug(face["name"]) == want for face in faces(p))]
    if not prints:
        return None
    paper = [p for p in prints if not p.get("digital")] or prints
    plain = [p for p in paper if not p.get("promo")] or paper
    first = min(plain, key=lambda p: p["released_at"])
    for face in faces(first):
        if slug(face["name"]) == want and face.get("artist"):
            return face["artist"]
    return first.get("artist")


def save(credits_file, credits):
    with open(credits_file, "w", encoding="utf-8", newline="\n") as f:
        json.dump(dict(sorted(credits.items())), f, ensure_ascii=False, indent=2)
        f.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    for entry in sorted(os.listdir(CONTENT)):
        dataset = os.path.join(CONTENT, entry)
        images = os.path.join(dataset, "images")
        if not os.path.isdir(images):
            continue

        credits_file = os.path.join(dataset, "artists.json")
        credits = {}
        if os.path.exists(credits_file):
            with open(credits_file, encoding="utf-8") as f:
                credits = json.load(f)

        names = card_names(dataset)
        slugs = sorted({os.path.splitext(f)[0] for f in os.listdir(images)
                        if os.path.splitext(f)[1].lower() in PICTURES and SLUG.match(os.path.splitext(f)[0])})
        wanted = [want for want in slugs if want not in credits]

        # Many names to a search, so a few hundred pictures are a few dozen requests rather than a thousand.
        named = [want for want in wanted if want in names]
        for i in range(0, len(named), 20):
            batch = named[i:i + 20]
            prints = printings(" or ".join(f'!"{names[want]}"' for want in batch))
            for want in batch:
                if artist := original_artist(want, prints):
                    credits[want] = artist
                    print(f"{want}: {artist}")
            if not args.dry_run:
                save(credits_file, credits)

        # What is left - a face's own name, a picture with no card file - one at a time.
        missing = []
        for want in wanted:
            if want in credits:
                continue
            name = names.get(want)
            if name is None:
                card = scryfall("cards/named", fuzzy=want.replace("-", " "))
                name = next((face["name"] for face in faces(card) if slug(face["name"]) == want), None) if card else None
            artist = original_artist(want, printings(f'name:"{name}"')) if name else None
            if artist:
                credits[want] = artist
                print(f"{want}: {artist}")
                if not args.dry_run:
                    save(credits_file, credits)
            else:
                missing.append(want)

        if missing:
            print(f"{entry}: no artist found for {', '.join(missing)}", file=sys.stderr)


if __name__ == "__main__":
    main()
