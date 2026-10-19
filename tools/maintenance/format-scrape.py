"""The cards a format actually plays, from the decks that are winning in it.

    python tools/maintenance/format-scrape.py standard [--days 60] [--events 60] [--decks 8]

Reads two sites, politely (one request at a time, a pause between them, every page kept in target/format-cache so a
second run asks for nothing):

    MTGGoldfish  the format's metagame page: every archetype with its share of the metagame, and each archetype's
                 deck, main and sideboard. A card's score is the sum of the shares of the archetypes that play it.
    MTGTop8      the format's recent events: the --decks best-placed decks of each of the --events newest events
                 held in the last --days days. A card's score is the share of those decks that play it.

Melee is not read: its decklists are behind a sign-in and its robots.txt closes the decklist and tournament-search
pages to crawlers.

Every name is checked against Scryfall and written as the card files spell it (card-list.py's mapping), and the
result is written to docs/formats/<format>.json, best-played first: the list `card-issues.py` makes issues from and
`next` hands out. Basic lands are left out; they are in every deck and in the engine already. The `score` is the mean
of the two shares, in percent: a card in a third of the winning decks scores near 33.
"""

import argparse
import datetime
import hashlib
import html
import importlib.util
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")
sys.dont_write_bytecode = True  # no __pycache__ left in tools/maintenance

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
CACHE = os.path.join(ROOT, "target", "format-cache")
OUT = os.path.join(ROOT, "docs", "formats")
AGENT = "cantrip-format-scrape/1.0 (+https://github.com/cantrip-rs/cantrip-rs)"
PAUSE = 0.8  # seconds between two requests to one site

# The format's name on each site. A new format is one line here.
FORMATS = {
    "standard": {"goldfish": "standard", "top8": "ST"},
    "pioneer": {"goldfish": "pioneer", "top8": "PI"},
    "modern": {"goldfish": "modern", "top8": "MO"},
    "legacy": {"goldfish": "legacy", "top8": "LE"},
    "vintage": {"goldfish": "vintage", "top8": "VI"},
    "pauper": {"goldfish": "pauper", "top8": "PAU"},
}
BASICS = {"Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes",
          "Snow-Covered Plains", "Snow-Covered Island", "Snow-Covered Swamp", "Snow-Covered Mountain",
          "Snow-Covered Forest"}


def load(name):
    """A sibling script (their names have hyphens) as a module."""
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), os.path.join(HERE, name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_last = {}


def fetch(url):
    """A page's text, from the cache when it has been asked for before."""
    path = os.path.join(CACHE, hashlib.sha1(url.encode()).hexdigest() + ".html")
    if os.path.isfile(path):
        with open(path, encoding="utf-8") as f:
            return f.read()
    site = url.split("/")[2]
    wait = PAUSE - (time.time() - _last.get(site, 0))
    if wait > 0:
        time.sleep(wait)
    request = urllib.request.Request(url, headers={"User-Agent": AGENT})
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            raw = response.read()
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:  # MTGTop8's deck exports are Windows-1252
            text = raw.decode("cp1252", "replace")
    except urllib.error.URLError as error:
        sys.exit(f"{url}: {error}")
    finally:
        _last[site] = time.time()
    os.makedirs(CACHE, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    return text


def deck_cards(text):
    """The card names of a decklist with counts (main and sideboard, 'sideboard' or 'SB:' marking the board)."""
    names = set()
    for line in text.splitlines():
        match = re.match(r"^(?:SB:\s*)?(\d+)x?\s+(.+?)\s*$", line.strip())
        if match:
            names.add(match.group(2))
    return names


def goldfish(site):
    """{card name: summed metagame percent}, and the metagame percent the archetypes read cover."""
    base = "https://www.mtggoldfish.com"
    page = fetch(f"{base}/metagame/{site}/full")
    scores, covered, archetypes = {}, 0.0, 0
    for tile in page.split("<div class='archetype-tile' id=")[1:]:
        link = re.search(r"card-image-tile-link-overlay' href='(/archetype/[^'#]+)", tile)
        share = re.search(r"META%\s*</div>\s*<div[^>]*>\s*([\d.]+)%", tile)
        if not link or not share:
            continue
        deck = re.search(r'name="deck_input\[deck\]" id="deck_input_deck" value="([^"]*)"', fetch(base + link.group(1)))
        if not deck:
            print(f"no decklist on {link.group(1)}", file=sys.stderr)
            continue
        percent = float(share.group(1))
        covered += percent
        archetypes += 1
        for name in deck_cards(html.unescape(deck.group(1))):
            scores[name] = scores.get(name, 0.0) + percent
    return scores, covered, archetypes


def top8(code, days, events, decks):
    """{card name: decks that play it}, and the number of decks read."""
    base = "https://www.mtgtop8.com"
    listing = fetch(f"{base}/format?f={code}")
    seen = []
    for event in re.findall(rf"event\?e=(\d+)&f={code}", listing):
        if event not in seen:
            seen.append(event)
    since = datetime.date.today() - datetime.timedelta(days=days)
    counts, total, used = {}, 0, 0
    for event in seen:
        if used >= events:
            break
        page = fetch(f"{base}/event?e={event}&f={code}")
        date = re.search(r"(\d\d)/(\d\d)/(\d\d)", page)
        if not date:
            continue
        held = datetime.date(2000 + int(date.group(3)), int(date.group(2)), int(date.group(1)))
        if held < since:
            continue
        used += 1
        ids = []
        for deck in re.findall(rf"\?e={event}&d=(\d+)&f={code}", page):
            if deck not in ids:
                ids.append(deck)
        for deck in ids[:decks]:
            total += 1
            for name in deck_cards(fetch(f"{base}/mtgo?d={deck}")):
                counts[name] = counts.get(name, 0) + 1
    return counts, total, used


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("format", choices=sorted(FORMATS))
    parser.add_argument("--days", type=int, default=60, help="how far back MTGTop8 events are read")
    parser.add_argument("--events", type=int, default=60, help="at most this many MTGTop8 events")
    parser.add_argument("--decks", type=int, default=8, help="the best-placed decks read from each event")
    args = parser.parse_args()
    site = FORMATS[args.format]

    gold, covered, archetypes = goldfish(site["goldfish"])
    print(f"MTGGoldfish: {archetypes} archetypes, {covered:.1f}% of the metagame", file=sys.stderr)
    top, decks, events = top8(site["top8"], args.days, args.events, args.decks)
    print(f"MTGTop8: {decks} decks from {events} events", file=sys.stderr)
    if not archetypes or not decks:
        sys.exit("a site returned nothing; its page may have changed")

    card_list = load("card-list")
    names = sorted(set(gold) | set(top))
    found, unknown = card_list.resolve(names)
    omenpaths = card_list.cubecobra()
    cards = {}
    for name, card in found.items():
        if card.get("layout") in card_list.NOT_CARDS:
            continue
        spelled = card_list.card_name(card, omenpaths)
        if spelled in BASICS:
            continue
        entry = cards.setdefault(spelled, {"name": spelled, "goldfish": 0.0, "top8": 0.0, "decks": 0})
        entry["goldfish"] += gold.get(name, 0.0) * 100 / covered
        entry["top8"] += top.get(name, 0) * 100 / decks
        entry["decks"] += top.get(name, 0)
    for entry in cards.values():
        entry["goldfish"] = round(entry["goldfish"], 2)
        entry["top8"] = round(entry["top8"], 2)
        entry["score"] = round((entry["goldfish"] + entry["top8"]) / 2, 2)
    ranked = sorted(cards.values(), key=lambda c: (-c["score"], c["name"]))

    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, args.format + ".json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump({
            "format": args.format,
            "generated": datetime.date.today().isoformat(),
            "sources": {
                "MTGGoldfish": {"archetypes": archetypes, "metagamePercent": round(covered, 1)},
                "MTGTop8": {"decks": decks, "events": events, "days": args.days},
            },
            "cards": ranked,
        }, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print(f"{len(ranked)} cards to {os.path.relpath(path, ROOT)}")
    for name in unknown:
        print(f"unknown card: {name}", file=sys.stderr)


if __name__ == "__main__":
    main()
