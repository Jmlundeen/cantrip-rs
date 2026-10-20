"""Scryfall's bulk data, cached once a day for every worktree of the repository: each card's current Oracle text and
its official rulings, by name.

    python tools/maintenance/scryfall.py "Card Name"     # each face's cost, type line, stats and Oracle text; rulings

    import scryfall
    scryfall.card("Card Name")   # {"name", "layout", "faces": [{"name", "mana_cost", "type_line", "power", "toughness",
                                 #  "loyalty", "defense", "color_indicator", "oracle_text"}], "rulings": [...]} or None
    scryfall.oracle_cards()      # every Oracle Cards object, for whole-pool reports (ability-index.py rank)

The bulk files go to target/scryfall in the main checkout (a worktree's git common directory), so the pool's slots
download them once between them. A day-old download is replaced; if Scryfall can't be reached, the last one serves, and
with none at all `card` returns None and says why on stderr.
"""

import datetime
import gzip
import json
import os
import subprocess
import sys
import urllib.request

HEADERS = {"User-Agent": "cantrip-maintenance/1.0", "Accept": "application/json"}
SKIPPED_LAYOUTS = {"token", "double_faced_token", "emblem", "art_series", "vanguard", "scheme", "planar"}
# What by-name.json keeps of each face; FORMAT changes whenever it does, so an older cache is rebuilt.
FACE_KEYS = ("mana_cost", "type_line", "power", "toughness", "loyalty", "defense", "color_indicator")
FORMAT = 2


def _main_checkout():
    """The repository's main working tree, which a worktree shares its .git directory with."""
    try:
        common = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                capture_output=True, text=True, check=True, cwd=os.path.dirname(os.path.abspath(__file__))).stdout.strip()
        if common:
            return os.path.dirname(os.path.normpath(common))
    except (OSError, subprocess.CalledProcessError):
        pass
    return os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))


CACHE = os.path.join(_main_checkout(), "target", "scryfall")


def bulk(kind):
    """The path of today's bulk file of a kind ("oracle-cards", "rulings"), downloading it if need be; else the newest
    one there is; else None."""
    os.makedirs(CACHE, exist_ok=True)
    today = os.path.join(CACHE, f"{kind}-{datetime.date.today():%Y%m%d}.jsonl.gz")
    if os.path.exists(today):
        return today
    try:
        meta = json.load(urllib.request.urlopen(urllib.request.Request(
            f"https://api.scryfall.com/bulk-data/{kind}", headers=HEADERS), timeout=30))
        print(f"downloading {meta['jsonl_download_uri']}", file=sys.stderr)
        with urllib.request.urlopen(urllib.request.Request(meta["jsonl_download_uri"], headers=HEADERS), timeout=300) as r:
            data = r.read()
        partial = today + ".part"
        with open(partial, "wb") as f:
            f.write(data)
        os.replace(partial, today)
        for old in os.listdir(CACHE):
            if old.startswith(kind + "-") and os.path.join(CACHE, old) != today:
                os.remove(os.path.join(CACHE, old))
        return today
    except (OSError, ValueError, KeyError) as e:
        older = sorted(f for f in os.listdir(CACHE) if f.startswith(kind + "-") and f.endswith(".jsonl.gz"))
        print(f"scryfall: could not download {kind} ({e}){'; using ' + older[-1] if older else ''}", file=sys.stderr)
        return os.path.join(CACHE, older[-1]) if older else None


def _lines(path):
    with gzip.open(path) as f:
        for line in f:
            if line.strip():
                yield json.loads(line)


def oracle_cards():
    """Every card object of the Oracle Cards file but tokens, emblems and the like."""
    path = bulk("oracle-cards")
    if path is None:
        return
    for d in _lines(path):
        if d.get("layout") not in SKIPPED_LAYOUTS:
            yield d


_index = None


def _load():
    """{lowercase name: card} for every card and face name, rulings attached, cached beside the bulk files."""
    global _index
    if _index is not None:
        return _index
    oracle, rulings = bulk("oracle-cards"), bulk("rulings")
    if oracle is None:
        _index = {}
        return _index
    stamp = f"v{FORMAT}+" + os.path.basename(oracle) + "+" + (os.path.basename(rulings) if rulings else "")
    built = os.path.join(CACHE, "by-name.json")
    try:
        with open(built, encoding="utf-8") as f:
            data = json.load(f)
        if data.get("stamp") == stamp:
            _index = data["cards"]
            return _index
    except (OSError, ValueError):
        pass
    by_oracle = {}
    if rulings:
        for r in _lines(rulings):
            by_oracle.setdefault(r["oracle_id"], []).append({"date": r["published_at"], "text": r["comment"]})
    # A paper card wins a name over a playtest, Alchemy or un-set card of the same name.
    lesser = {"funny": 1, "playtest": 2, "memorabilia": 3, "alchemy": 4}
    cards, rank = {}, {}
    for d in _lines(oracle):
        if d.get("layout") in SKIPPED_LAYOUTS:
            continue
        faces = d.get("card_faces") or [d]
        card = {
            "name": d["name"],
            "layout": d.get("layout", ""),
            "faces": [{key: f.get(key, d.get(key, "") if key in ("mana_cost", "type_line") and len(faces) == 1 else "")
                       for key in FACE_KEYS} | {"name": f["name"], "oracle_text": f.get("oracle_text", "")}
                      for f in faces],
            "rulings": sorted(by_oracle.get(d.get("oracle_id") or faces[0].get("oracle_id"), []), key=lambda r: r["date"]),
        }
        order = lesser.get(d.get("set_type"), 0)
        for name in {d["name"]} | {f["name"] for f in faces}:
            key = name.lower()
            if key not in cards or order < rank[key]:
                cards[key], rank[key] = card, order
    partial = built + ".part"
    with open(partial, "w", encoding="utf-8") as f:
        json.dump({"stamp": stamp, "cards": cards}, f)
    os.replace(partial, built)
    _index = cards
    return _index


def card(name):
    return _load().get(name.lower())


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    found = card(sys.argv[1])
    if found is None:
        sys.exit(f"no card named {sys.argv[1]!r} on Scryfall")
    if found.get("layout") not in (None, "", "normal"):
        print(f"layout: {found['layout']}\n")
    for face in found["faces"]:
        stats = "/".join(str(face[k]) for k in ("power", "toughness") if face.get(k))
        stats = stats or (f"loyalty {face['loyalty']}" if face.get("loyalty") else f"defense {face['defense']}" if face.get("defense") else "")
        header = "  ".join(x for x in (face["name"], face.get("mana_cost", ""), face.get("type_line", ""), stats) if x)
        indicator = f"\ncolor indicator: {''.join(face['color_indicator'])}" if face.get("color_indicator") else ""
        print(f"{header}{indicator}\n{face['oracle_text']}\n")
    for r in found["rulings"]:
        print(f"{r['date']}  {r['text']}")


if __name__ == "__main__":
    main()
