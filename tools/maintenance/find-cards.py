"""Implemented cards to build a scenario's board from, by what they are: the plainest first.

    python tools/maintenance/find-cards.py --type creature --toughness 4
    python tools/maintenance/find-cards.py --type creature --vanilla --power 2
    python tools/maintenance/find-cards.py --type artifact --mv 2 --color colorless
    python tools/maintenance/find-cards.py --type creature --keyword flying --text "can't block"

Every card in content/ is implemented. A scenario wants the plainest card that has what it tests and nothing else that
could change the result, so the list puts first the cards with the least rules text (a vanilla creature, a Mox), then
the cheapest. `--vanilla` keeps cards with no rules text but the core keywords asked for with `--keyword`.
"""

import argparse
import json
import os
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
COLORS = {"white": "W", "blue": "U", "black": "B", "red": "R", "green": "G"}


def mana_value(cost):
    """CR 202.3: each symbol's value - a number its number, X 0, a hybrid with a number its number, any other 1."""
    total = 0
    for symbol in re.findall(r"\{([^}]+)\}", cost or ""):
        parts = symbol.split("/")
        if symbol.isdigit():
            total += int(symbol)
        elif symbol in ("X", "Y", "Z"):
            continue
        elif parts[0].isdigit():
            total += int(parts[0])
        else:
            total += 1
    return total


def colors_of(card):
    if card.get("colorIndicator"):
        return {COLORS[c.lower()] for c in card["colorIndicator"]}
    return {c for c in "WUBRG" if c in re.sub(r"\{[^}]*P[^}]*\}", lambda m: m.group(0).replace("P", ""), card.get("manaCost", ""))}


def rules_text(card):
    """Oracle text without reminder text, every face's (an adventure's spell is part of what the card does)."""
    faces = [card] + [card[k] for k in ("backFace", "adventure", "omen") if isinstance(card.get(k), dict)]
    faces += [h for h in card.get("halves", []) if isinstance(h, dict)]
    text = "\n".join(f.get("oracleText", "") for f in faces if f.get("oracleText"))
    return re.sub(r"\([^)]*\)", "", text).strip()


def load():
    for dataset in sorted(os.listdir(os.path.join(ROOT, "content"))):
        folder = os.path.join(ROOT, "content", dataset, "cards")
        if not os.path.isdir(folder):
            continue
        for file in os.listdir(folder):
            if file.endswith(".json"):
                text = open(os.path.join(folder, file), encoding="utf-8-sig").read()
                try:
                    yield json.loads("\n".join(l for l in text.split("\n") if not l.strip().startswith("//")))
                except ValueError:
                    continue


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--type", action="append", default=[], help="a word of the type line (creature, artifact, Elf...)")
    parser.add_argument("--power", type=int)
    parser.add_argument("--toughness", type=int)
    parser.add_argument("--mv", type=int, help="mana value")
    parser.add_argument("--max-mv", type=int)
    parser.add_argument("--color", help="white, blue, black, red, green or colorless (a card of exactly that color)")
    parser.add_argument("--keyword", action="append", default=[], help="a core keyword it has (flying, first strike...)")
    parser.add_argument("--text", action="append", default=[], help="words its Oracle text contains")
    parser.add_argument("--vanilla", action="store_true", help="no rules text but the --keyword ones")
    parser.add_argument("-n", type=int, default=15)
    args = parser.parse_args()

    found = []
    for card in load():
        type_line = card.get("type", "").lower()
        text = rules_text(card)
        if any(t.lower() not in type_line for t in args.type):
            continue
        if args.power is not None and str(card.get("power")) != str(args.power):
            continue
        if args.toughness is not None and str(card.get("toughness")) != str(args.toughness):
            continue
        mv = mana_value(card.get("manaCost"))
        if (args.mv is not None and mv != args.mv) or (args.max_mv is not None and mv > args.max_mv):
            continue
        if args.color:
            want = set() if args.color == "colorless" else {COLORS[args.color]}
            if colors_of(card) != want:
                continue
        keywords = {k.lower() for k in card.get("keywords", [])}
        if any(k.replace(" ", "").lower() not in keywords for k in args.keyword):
            continue
        if any(t.lower() not in text.lower() for t in args.text):
            continue
        if args.vanilla:
            rest = [line for line in text.split("\n") if line.strip()]
            asked = {k.lower() for k in args.keyword}
            if any(any(p.strip().lower() not in asked for p in line.split(",")) for line in rest):
                continue
        found.append((len(text), mv, card["name"], card))
    found.sort(key=lambda f: (f[0], f[1], f[2]))
    for _, mv, name, card in found[: args.n]:
        stats = f" {card['power']}/{card['toughness']}" if "power" in card else ""
        text = rules_text(card).replace("\n", " / ")
        print(f"{name}  {card.get('manaCost', '')}  {card.get('type', '')}{stats}" + (f"  - {text[:90]}" if text else ""))
    if not found:
        print("none")
    elif len(found) > args.n:
        print(f"... {len(found) - args.n} more (-n)")


if __name__ == "__main__":
    main()
