"""Fallback card art from Scryfall, for faces mtgpics has no art-only picture of (recent sets, some name searches).

    python tools/scryfall-art/fetch_art.py <out-dir> < names.txt        names one a line (missing-art.py's output)
    python tools/scryfall-art/fetch_art.py <out-dir> --name "Torch the Tower"

Writes <out-dir>/<slug>.jpg (the client's CardArt.Slug) from the printing's `art_crop` image - the art alone - and prints one
line a card: `OK`, `OK PROMO-ONLY`, `NOART` or `ERR`. A file that already exists is never replaced.

Choice, as in the mtgpics fetcher: the earliest regular printing wins (promo, Secret Lair, box topper, digital-only and
other non-paper printings are skipped, token/"full art" or borderless variants lose to a normal frame of the same card).
Scryfall asks for a User-Agent and 50-100 ms between requests; this sleeps 150 ms.
"""
import json, os, re, sys, time, unicodedata, urllib.error, urllib.parse, urllib.request

API = "https://api.scryfall.com/"
UA = {"User-Agent": "CantripArtFetcher/1.0", "Accept": "application/json;q=0.9,*/*;q=0.8"}


def slug(name):
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c)).lower().replace("'", "").replace("’", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", folded)).strip("-")


def get(url):
    for i in range(4):
        try:
            time.sleep(.15)
            return urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=30).read()
        except urllib.error.HTTPError as e:
            if e.code == 404:
                return None
            if e.code not in (429, 500, 502, 503):
                raise
            time.sleep(2 * (i + 1))
    raise RuntimeError("Scryfall kept failing: " + url)


def prints(name):
    """Every paper printing of the card, the face called `name` among its faces."""
    first = get(API + "cards/named?exact=" + urllib.parse.quote(name))
    if first is None:
        first = get(API + "cards/named?fuzzy=" + urllib.parse.quote(name))
    if first is None:
        return []
    uri = json.loads(first)["prints_search_uri"]
    cards = []
    while uri:
        page = json.loads(get(uri))
        cards += page["data"]
        uri = page.get("next_page")
    return cards


def crop_of(card, name):
    if "image_uris" in card:
        return card["image_uris"].get("art_crop")
    for face in card.get("card_faces", []):
        if slug(face["name"]) == slug(name) and "image_uris" in face:
            return face["image_uris"].get("art_crop")
    return None


def regular(card):
    return (not card.get("digital") and not card.get("promo") and not card.get("full_art")
            and card.get("set_type") not in ("promo", "memorabilia", "token", "funny", "minigame")
            and card.get("border_color") not in ("gold", "borderless")
            and not {"showcase", "extendedart"} & set(card.get("frame_effects", [])))


def fetch(name, out):
    target = os.path.join(out, slug(name) + ".jpg")
    cards = [c for c in prints(name) if crop_of(c, name)]
    if not cards:
        return "NOART " + name
    pool = [c for c in cards if regular(c)] or \
           [c for c in cards if not c.get("digital") and not c.get("promo")]
    plain = bool(pool)
    pool = pool or cards
    pick = min(pool, key=lambda c: (c["released_at"], c.get("collector_number", "")))
    data = get(crop_of(pick, name))
    with open(target, "wb") as f:
        f.write(data)
    return f"OK{'' if plain else ' PROMO-ONLY'} {name} {pick['set']}/{pick['collector_number']} {pick['released_at']} {len(data)} bytes"


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    out = args[0]
    names = [args[args.index("--name") + 1]] if "--name" in args else [n.strip() for n in sys.stdin.read().splitlines() if n.strip()]
    os.makedirs(out, exist_ok=True)
    for name in names:
        if os.path.exists(os.path.join(out, slug(name) + ".jpg")):
            print("KEPT " + name, flush=True)
            continue
        try:
            print(fetch(name, out), flush=True)
        except Exception as e:
            print(f"ERR {name} {e}", flush=True)


if __name__ == "__main__":
    main()
