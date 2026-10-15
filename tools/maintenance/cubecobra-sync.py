"""Keeps a cube list in docs/cube-lists/ in step with the cube it is kept from on CubeCobra, one cube a run.

    python tools/maintenance/cubecobra-sync.py <cube>               sync that cube's list
    python tools/maintenance/cubecobra-sync.py <cube> --check       say what would change, write nothing
    python tools/maintenance/cubecobra-sync.py <cube> --as <List>   a new cube's list is <List>.txt, not its short id

<cube> is a list already kept from CubeCobra (a name in docs/cube-lists/cubecobra.json), or any cube on CubeCobra: its
short id - the unique name in its address, cubecobra.com/cube/list/<id>, which CubeCobra matches case and all - its
long id, or the address itself. A cube not kept yet is fetched whole into a new list named for its short id, and
docs/cube-lists/cubecobra.json records where it came from, so the next run needs only the name.

docs/cube-lists/cubecobra.json maps list name to CubeCobra cube id. A list it does not name is kept by hand, and a cube
whose list would land on one is refused rather than written over it: name it otherwise with --as, or map the hand-kept
list in the file first if the cube really is its source.

The list is rewritten whole from the cube's mainboard, in CubeCobra's own order, one name a line, a card the cube holds
twice written twice; its maybeboard and basics are left out. Names are CubeCobra's card names: the front face of a
double-faced card, "A // B" for a split card - the spelling the card files and the other lists already use.

It prints the names added (+) and removed (-), then two lines for a caller to read: "list: <List>", the list this run
kept, and "changed: yes" or "changed: no". A card that leaves a list keeps its card file: content is a dataset of
cards, not of the cube.
"""

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LISTS = os.path.join(ROOT, "docs", "cube-lists")
SOURCES = os.path.join(LISTS, "cubecobra.json")
CUBE_JSON = "https://cubecobra.com/cube/api/cubeJSON/{}"
AGENT = {"User-Agent": "cantrip-cubecobra-sync"}
NOT_FOUND = " (short ids are case-sensitive: copy it from the cube's address)"


def load_sources():
    if not os.path.exists(SOURCES):
        return {}
    with open(SOURCES, encoding="utf-8") as f:
        return json.load(f)


def save_sources(sources):
    with open(SOURCES, "w", encoding="utf-8", newline="\n") as f:
        json.dump(dict(sorted(sources.items())), f, ensure_ascii=False, indent=2)
        f.write("\n")


def cube_id(text):
    """The id in a cubecobra.com address (/cube/list/<id>, /cube/overview/<id>, ...), or the text as given."""
    match = re.search(r"cubecobra\.com/cube/[a-z]+/([^/?#\s]+)", text)
    return match.group(1) if match else text.strip()


def fetch_cube(cube):
    """The cube's JSON, or exit saying why there is none."""
    request = urllib.request.Request(CUBE_JSON.format(urllib.parse.quote(cube)), headers=AGENT)
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        sys.exit(f"cube {cube}: {error}" + (NOT_FOUND if error.code == 404 else ""))
    except urllib.error.URLError as error:
        sys.exit(f"cube {cube}: {error}")
    try:
        return json.loads(body)
    except json.JSONDecodeError:
        # A cube that does not exist may also answer 200 with "Cube not found." as plain text.
        sys.exit(f"cube {cube}: {body.strip()[:80] or 'empty answer'}{NOT_FOUND}")


PRINTED_NAMES = os.path.join(LISTS, "printed-names.json")
RENAMED_SETS = {"om1"}  # Through the Omenpaths: reprints under new names, never used for anything here
SCRYFALL_PRINTINGS = "https://api.scryfall.com/cards/search?q=oracleid%3A{}&unique=prints&order=released&dir=asc"


def original_name(details):
    """The card's name as the original printing has it. A cube may hold a Through the Omenpaths reprint, which carries
    a new name for an existing card ("Makdee and Itla, Skysnarers" for Spider-Woman, Stunning Savior); the card files
    and every list here use the original card's name, so the reprint's name is mapped through its oracle id to the
    earliest printing's name, and the answer is kept in docs/cube-lists/printed-names.json so later runs are offline.
    A card first printed there keeps its name, since it is the original."""
    if details.get("set") not in RENAMED_SETS:
        return details["name"]
    known = {}
    if os.path.exists(PRINTED_NAMES):
        with open(PRINTED_NAMES, encoding="utf-8") as f:
            known = json.load(f)
    if details["name"] in known:
        return known[details["name"]]
    request = urllib.request.Request(SCRYFALL_PRINTINGS.format(details["oracle_id"]),
                                     headers={"User-Agent": "cantrip/1.0", "Accept": "application/json"})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            printings = json.load(response)["data"]
    except (urllib.error.URLError, KeyError, json.JSONDecodeError) as error:
        sys.exit(f"{details['name']}: cannot find its original printing on Scryfall ({error})")
    original = printings[0]["name"] if printings else details["name"]
    known[details["name"]] = original
    with open(PRINTED_NAMES, "w", encoding="utf-8", newline="\n") as f:
        json.dump(dict(sorted(known.items())), f, indent=2, ensure_ascii=False)
        f.write("\n")
    return original


def read_list(path):
    """The names a list holds, in order, as CubeList reads them: # comments, counts, nothing after Sideboard."""
    names = []
    if not os.path.exists(path):
        return names
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.lower().startswith("sideboard"):
                break
            count, _, rest = line.partition(" ")
            if count.rstrip("x").isdigit() and rest:
                names.extend([rest.strip()] * int(count.rstrip("x")))
            else:
                names.append(line)
    return names


def write_list(path, names, cube):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(f"# mainboard of https://cubecobra.com/cube/list/{cube}, by tools/maintenance/cubecobra-sync.py\n")
        for name in names:
            f.write(name + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("cube", help="a list kept from CubeCobra, or a CubeCobra cube's short id, long id or address")
    parser.add_argument("--check", action="store_true", help="say what would change and write nothing")
    parser.add_argument("--as", dest="name", metavar="LIST", help="a new cube's list name (default: its short id)")
    args = parser.parse_args()

    sources = load_sources()
    if args.cube in sources:
        name, wanted = args.cube, sources[args.cube]
        if args.name and args.name != name:
            sys.exit(f"{name} is already kept from cube {wanted}; --as only names a new one")
    else:
        name, wanted = None, cube_id(args.cube)

    cube = fetch_cube(wanted)
    short, long = cube.get("shortId") or wanted, cube.get("id")
    if name is None:
        # A cube kept under another name, asked for by its id: that list, not a second copy of it.
        kept = [list_name for list_name, source in sources.items() if source in (short, long, wanted)]
        name = args.name or (kept[0] if kept else short)
        if kept and name not in kept:
            sys.exit(f"cube {short} is already kept as {kept[0]}")
        if name not in sources and os.path.exists(os.path.join(LISTS, name + ".txt")):
            sys.exit(f"{name}.txt is kept by hand, not from CubeCobra; give the cube another name with --as, or map "
                     f"{name} to it in {os.path.relpath(SOURCES, ROOT)} first if it really is the list's source")
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", name):
            sys.exit(f"'{name}' is not a usable list name; give one with --as")

    path = os.path.join(LISTS, name + ".txt")
    remote = [original_name(card["details"]) for card in cube["cards"]["mainboard"]]
    if not remote:
        sys.exit(f"cube {short}: its mainboard is empty")
    local = read_list(path)
    added = sorted(set(remote) - set(local))
    removed = sorted(set(local) - set(remote))
    fresh = name not in sources
    print(f"{name} <- {short} ({cube.get('name', short)}): {len(remote)} cards, "
          + ("new list" if fresh else f"{len(added)} added, {len(removed)} removed"))
    for card in added:
        print(f"  + {card}")
    for card in removed:
        print(f"  - {card}")

    changed = remote != local or fresh
    if changed and not args.check:
        write_list(path, remote, short)
        if fresh:
            sources[name] = short
            save_sources(sources)
    print(f"list: {name}")
    print("changed: " + ("yes" if changed else "no"))


if __name__ == "__main__":
    main()
