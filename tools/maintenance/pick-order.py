"""Writes a pick order for each cube on docs/cube-lists: <Cube>.picks.json beside <Cube>.txt, every card of the cube
best first, which the draft clock and the bots' drafter follow.

    python tools/maintenance/pick-order.py [Cube ...]              rank from the ratings kept, fetching only cards never rated
    python tools/maintenance/pick-order.py --refresh [Cube ...]    fetch every rating again first

Card quality by consensus, from what drafters actually took:

- 17lands' public pick data for Arena's Powered Cube - every pick, with the pack it was taken from. A card taken over
  another in the same pack beat it; a Bradley-Terry fit over the first eight picks of each pack (before colours have
  much say) gives every card a strength.
- CubeCobra's draft Elo, for the cards Arena's cube does not have. Elo is mapped onto the 17lands strengths by a straight
  line fitted over the cards both know (they agree closely), so the two rank on one scale.

Every rating fetched is kept in docs/cube-lists/card-ratings.json - all of Arena's cube, not only the cards on our
lists - so a cube that changes is ranked again from it, and only a card nobody has rated yet is looked up (on
CubeCobra, card by card). It is scraped data, so it is gitignored and stays on this machine; the pick orders are
committed. Without it, the next run fetches everything again. The 17lands data is ~100 MB and is fetched only with --refresh, into %TEMP%/cantrip-pick-order.
"""

import collections
import csv
import gzip
import json
import math
import os
import re
import statistics
import sys
import tempfile
import time
import unicodedata
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CUBES = os.path.join(ROOT, "docs", "cube-lists")
RATINGS = os.path.join(CUBES, "card-ratings.json")
CACHE = os.path.join(tempfile.gettempdir(), "cantrip-pick-order")
SEVENTEEN = "https://17lands-public.s3.amazonaws.com/analysis_data/draft_data/draft_data_public.Cube_-_Powered.{}.csv.gz"
FORMATS = ["PremierDraft", "TradDraft"]
CUBECOBRA_CUBE = "https://cubecobra.com/cube/api/cubeJSON/LSVCube"
CUBECOBRA_CARD = "https://cubecobra.com/tool/card/{}"
EARLY_PICKS = 8      # picks 1-8 of each pack
TRUSTED_SEEN = 300   # a 17lands strength from fewer sightings than this gives way to Elo, where there is one
AGENT = {"User-Agent": "cantrip-pick-order"}


def norm(name):
    """Names as both sources and the cube lists spell them: no accents, no case, the front face of a split card."""
    plain = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode()
    return plain.lower().split(" // ")[0].strip()


def cube_cards(path):
    """The cube's names, in the decklist format CubeList reads: # comments, counts, nothing after Sideboard."""
    names = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.lower().startswith("sideboard"):
                break
            m = re.match(r"^(\d+)\s+(.*)$", line)
            name = m.group(2) if m else line
            if name not in names:
                names.append(name)
    return names


def fetch(url, file, fresh=False):
    path = os.path.join(CACHE, file)
    if fresh or not os.path.exists(path):
        os.makedirs(CACHE, exist_ok=True)
        print(f"fetching {url}", file=sys.stderr)
        with urllib.request.urlopen(urllib.request.Request(url, headers=AGENT), timeout=600) as response:
            data = response.read()
        with open(path, "wb") as f:
            f.write(data)
    return path


def seventeen_lands(fresh):
    """Bradley-Terry strengths (log scale) and sightings, by card name as 17lands spells it."""
    wins = collections.Counter()
    pairs = collections.Counter()
    seen = collections.Counter()
    for fmt in FORMATS:
        path = fetch(SEVENTEEN.format(fmt), f"17lands.{fmt}.csv.gz", fresh)
        with gzip.open(path, "rt", encoding="utf-8") as f:
            reader = csv.reader(f)
            header = next(reader)
            columns = [(i, c[len("pack_card_"):]) for i, c in enumerate(header) if c.startswith("pack_card_")]
            pick_at, number_at = header.index("pick"), header.index("pick_number")
            for row in reader:
                if int(row[number_at]) >= EARLY_PICKS:  # pick_number counts from 0
                    continue
                taken = row[pick_at]
                seen[taken] += 0
                for i, card in columns:
                    if row[i] == "0":
                        continue
                    seen[card] += 1
                    if card != taken:
                        wins[taken] += 1
                        pairs[tuple(sorted((taken, card)))] += 1

    met = collections.defaultdict(list)
    for (a, b), n in pairs.items():
        met[a].append((b, n))
        met[b].append((a, n))
    cards = list(seen)
    strength = {card: 1.0 for card in cards}
    for _ in range(300):
        # Minorisation-maximisation, with half a win and a game against an average card as a prior.
        updated = {}
        for card in cards:
            games = sum(n / (strength[card] + strength[other]) for other, n in met[card]) + 1 / (strength[card] + 1)
            updated[card] = (wins[card] + 0.5) / games
        mean = math.exp(statistics.fmean(math.log(v) for v in updated.values()))
        strength = {card: v / mean for card, v in updated.items()}
    return {card: {"strength": round(math.log(strength[card]), 4), "seen": seen[card]} for card in cards}


def cubecobra_cube(fresh):
    """CubeCobra's Elo for every card of its LSV cube, in one call."""
    with open(fetch(CUBECOBRA_CUBE, "cubecobra.LSVCube.json", fresh), encoding="utf-8") as f:
        cards = json.load(f)["cards"]["mainboard"]
    return {card["details"]["name"]: round(card["details"]["elo"], 1) for card in cards if card["details"].get("elo")}


def cubecobra_card(name, fresh):
    """CubeCobra's Elo for one card, from its page; None if it has none."""
    slug = re.sub(r"[^a-z0-9]+", "-", norm(name)).strip("-")
    path = fetch(CUBECOBRA_CARD.format(urllib.parse.quote(name.split(" // ")[0])), f"cubecobra.card.{slug}.html", fresh)
    time.sleep(0.5)
    with open(path, encoding="utf-8") as f:
        m = re.search(r'window\.reactProps = \{"card":\{"elo":([0-9.]+)', f.read())
    return round(float(m.group(1)), 1) if m else None


def load_ratings():
    if not os.path.exists(RATINGS):
        return {"17lands": {}, "cubecobra": {}}
    with open(RATINGS, encoding="utf-8") as f:
        return json.load(f)


def save_ratings(ratings):
    ratings["17lands"] = dict(sorted(ratings["17lands"].items()))
    ratings["cubecobra"] = dict(sorted(ratings["cubecobra"].items()))
    with open(RATINGS, "w", encoding="utf-8", newline="\n") as f:
        json.dump(ratings, f, ensure_ascii=False, indent=1)
        f.write("\n")


def main(args):
    refresh = "--refresh" in args
    cubes = [a for a in args if not a.startswith("--")] or sorted(f[:-4] for f in os.listdir(CUBES) if f.endswith(".txt"))
    lists = {cube: cube_cards(os.path.join(CUBES, cube + ".txt")) for cube in cubes}
    everything = sorted({name for cards in lists.values() for name in cards})

    ratings = load_ratings()
    if refresh or not ratings["17lands"]:
        ratings["17lands"] = seventeen_lands(refresh)
        ratings["17landsSource"] = "Arena Cube - Powered, " + " and ".join(FORMATS) + f", picks 1-{EARLY_PICKS} of each pack, " + time.strftime("%Y-%m-%d")
    if refresh or not ratings["cubecobra"]:
        ratings["cubecobra"].update(cubecobra_cube(refresh))
        ratings["cubecobraSource"] = "CubeCobra card Elo, " + time.strftime("%Y-%m-%d")
    known = {norm(name) for name in ratings["cubecobra"]}
    for name in everything:
        if norm(name) not in known or (refresh and name in ratings["cubecobra"]):
            ratings["cubecobra"][name] = cubecobra_card(name, refresh)
    save_ratings(ratings)

    lands = {norm(name): (r["strength"], r["seen"]) for name, r in ratings["17lands"].items()}
    elo = {norm(name): e for name, e in ratings["cubecobra"].items() if e is not None}
    both = [(elo[k], lands[k][0]) for k in elo if k in lands and lands[k][1] >= TRUSTED_SEEN]
    slope, intercept = statistics.linear_regression([e for e, _ in both], [s for _, s in both])
    fit = statistics.correlation([e for e, _ in both], [s for _, s in both])
    print(f"{len(both)} cards in both sources, correlation {fit:.3f}", file=sys.stderr)

    def score(name):
        key = norm(name)
        if key in lands and (lands[key][1] >= TRUSTED_SEEN or key not in elo):
            return lands[key][0]
        if key in elo:
            return slope * elo[key] + intercept
        return None

    for cube, cards in lists.items():
        scored = [(score(name), name) for name in cards]
        unknown = [name for s, name in scored if s is None]
        if unknown:
            print(f"{cube}: no rating for {', '.join(unknown)}; ranked last", file=sys.stderr)
        ordered = sorted(scored, key=lambda pair: (pair[0] is None, -(pair[0] or 0), pair[1]))
        out = {
            "cube": cube,
            "source": "card-ratings.json by tools/maintenance/pick-order.py: 17lands Arena Powered Cube picks (Bradley-Terry), "
                      "CubeCobra draft Elo for cards Arena lacks, mapped onto the same scale",
            "picks": [name for _, name in ordered],
        }
        path = os.path.join(CUBES, cube + ".picks.json")
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            json.dump(out, f, ensure_ascii=False, indent=2)
            f.write("\n")
        print(f"wrote {os.path.relpath(path, ROOT)}: {len(ordered)} cards", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv[1:])
