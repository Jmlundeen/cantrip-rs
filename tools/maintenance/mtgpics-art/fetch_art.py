"""Fetch card art from mtgpics.com: the original, non-promo printing's art-only picture, at the best resolution.

    python tools/maintenance/mtgpics-art/fetch_art.py <out-dir> [--install] < names.txt      names one a line (missing-art.py's output)
    python tools/maintenance/mtgpics-art/fetch_art.py <out-dir> --name "Eternal Witness"

Writes <out-dir>/<slug>.jpg (the client's CardArt.Slug: lower case, hyphens, no apostrophes) and prints one line a card:
`OK`, `OK PROMO-ONLY` (mtgpics holds no regular printing's art), `NOART` or `ERR`. With --install <out-dir> is
content/mtg/images, and a file that exists is replaced only by a larger picture.

How mtgpics is laid out (found by hand, no API):
  POST results?zbob=1  cardtitle_search=<name>   one hit answers `<script>location="card?ref=<ref>"`, several a list page
  card?ref=<ref>                                 <title> is the card name (split cards "A/B"); links art?gamerid=<ref>
  art?gamerid=<ref>                              every distinct art of that card name, thumbnails pics/art_th_big/<set>/<n>.jpg
  pics/art/<set>/<n>.jpg                         the art alone, full size  <- what we want
  sets, set?set=<id>                             set code (logo file name), release date, and the set's title
The server answers 500 now and then, and art?gamerid=<ref> 500s for some refs; both are retried or skipped.

Choice: drop promo sets (title matches PROMO below, or the set is unknown), take the earliest release (ties: set id),
then swap in a larger picture only if it is visually the same one (32x32 grey difference), so a reprint's crop or
a higher-resolution scan wins but a different illustration never does. Portrait art (planeswalkers, full-art frames)
is kept; the client crops.
"""
import html, io, os, re, shutil, sys, time, unicodedata, urllib.error, urllib.parse, urllib.request
from PIL import Image, ImageChops, ImageStat

BASE = "https://www.mtgpics.com/"
UA = {"User-Agent": "Mozilla/5.0"}
PROMO = re.compile(r"Promo|Secret Lair|Box Topper|Special Guests|Source Material|Anniversary|Judge|Bonus|Prerelease|"
                   r"Showcase|Gift|Convention|Celebration|Playtest|Token", re.I)
MONTHS = "january february march april may june july august september october november december".split()
ART = r"pics/art(?:_th|_th_big)?/(\w+/\w+)\.jpg"


def get(url, data=None):
    req = urllib.request.Request(url if url.startswith("http") else BASE + url, data=data, headers=UA)
    return urllib.request.urlopen(req, timeout=30).read()


def retry(f, tries=4):
    for i in range(tries):
        try:
            return f()
        except urllib.error.HTTPError as e:
            if e.code not in (500, 502, 503):
                raise
            time.sleep(2 * (i + 1))
    return f()


def slug(name):
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c)).lower().replace("'", "").replace("’", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", folded)).strip("-")


def text(url, data=None):
    return retry(lambda: get(url, data)).decode("utf8", "ignore")


def load_sets():
    page = text("sets")
    sets = {}
    for m in re.finditer(r"href=set\?set=(\d+)><img src=graph/sets/logos/(\w+)\.png></a>(.*?)</table>", page, re.S):
        d = re.search(r"class=G12>([^<]+)<", m.group(3))
        sets[m.group(2)] = (int(m.group(1)), d.group(1).strip() if d else "")
    return sets


SETS = {}
TITLES = {}


def release(code):
    m = re.match(r"(\w+) (\d{4})", SETS[code][1])
    return (int(m.group(2)), MONTHS.index(m.group(1).lower()) + 1) if m and m.group(1).lower() in MONTHS else (9999, 0)


def is_promo(code):
    if code not in SETS:
        return True
    if code not in TITLES:
        TITLES[code] = re.search(r"<title>(.*?) - ", text("set?set=%d" % SETS[code][0])).group(1)
    return bool(PROMO.search(TITLES[code]))


def page_title(page):
    m = re.search(r"<title>(.*?) - ", page)
    return html.unescape(m.group(1)).strip() if m else ""


def is_card(title, name):
    n = slug(name)
    return slug(title) == n or n in [slug(half) for half in title.split("/")]


def card_refs(name):
    def search():
        try:
            q = "results?type_search=&costx=1&endx=1&forx=1&cardtitle_search=" + urllib.parse.quote(name) + "&graph_aff=1"
            return text(q)
        except Exception:
            return text("results?zbob=1", urllib.parse.urlencode({"cardtitle_search": name}).encode())
    page = search()
    m = re.match(r'\s*<script>location="card\?ref=(\w+)"', page)
    return [m.group(1)] if m else list(dict.fromkeys(re.findall(r"href=/card\?ref=(\w+)", page)))


def art_paths(name):
    paths = []
    for ref in card_refs(name)[:60]:
        card = text("card?ref=" + ref)
        if not is_card(page_title(card), name):
            continue
        for g in dict.fromkeys(re.findall(r"art\?gamerid=(\w+)", card) + [ref]):
            try:
                paths += re.findall(ART, text("art?gamerid=" + g))
            except Exception:
                pass
        paths += re.findall(ART, card)
        if paths:
            break
    return list(dict.fromkeys(paths))


def thumb(data):
    return Image.open(io.BytesIO(data)).convert("L").resize((32, 32))


def same_picture(a, b):
    return ImageStat.Stat(ImageChops.difference(a, b)).mean[0] < 30


def fetch(name, out, install=False):
    paths = art_paths(name)
    if not paths:
        return "NOART " + name
    code = lambda p: p.split("/")[0]
    regular = [p for p in paths if code(p) in SETS and not is_promo(code(p))]
    pool = sorted(regular or paths, key=lambda p: (release(code(p)), SETS[code(p)][0]) if code(p) in SETS else ((9999, 0), 0))
    data = retry(lambda: get(f"pics/art/{pool[0]}.jpg"))
    best = (Image.open(io.BytesIO(data)).size, data, pool[0])
    first = thumb(data)
    for p in pool[1:]:
        try:
            d = retry(lambda: get(f"pics/art/{p}.jpg"))
            size = Image.open(io.BytesIO(d)).size
        except Exception:
            continue
        if size[0] * size[1] > best[0][0] * best[0][1] and same_picture(first, thumb(d)):
            best = (size, d, p)
        time.sleep(.2)
    target = os.path.join(out, slug(name) + ".jpg")
    if install and os.path.exists(target):
        have = Image.open(target).size
        if have[0] * have[1] >= best[0][0] * best[0][1]:
            return f"KEPT {name} existing {have[0]}x{have[1]}"
    with open(target, "wb") as f:
        f.write(best[1])
    return f"OK{'' if regular else ' PROMO-ONLY'} {name} original={pool[0]} used={best[2]} {best[0][0]}x{best[0][1]}"


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    out, install = args[0], "--install" in args
    names = [args[args.index("--name") + 1]] if "--name" in args else [n.strip() for n in sys.stdin.read().splitlines() if n.strip()]
    SETS.update(load_sets())
    os.makedirs(out, exist_ok=True)
    for name in names:
        try:
            print(fetch(name, out, install), flush=True)
        except Exception as e:
            print(f"ERR {name} {e}", flush=True)
        time.sleep(.4)


if __name__ == "__main__":
    main()
