"""Find the code a card's file shares with other cards' files: SKILL.md "Share what is the same".

    python tools/maintenance/card-dupes.py "Card Name" ["Other Card" ...]

For each function in the card's file, lists every other card file with a function whose body is the same once
comments and whitespace are dropped and the names that only pick a kind (`CounterKind::PlusOne`, `Subtype::Angel`,
`Color::Red`, numbers) are masked. A match is a candidate for one shared function in `cantrip-mechanics`, moved in the
same change - or two bodies that only look alike (one that reads "you" and one that reads "each opponent" are not the
same): read both before sharing. A body of one or two statements that only calls the engine is not reported.
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import importlib.util  # noqa: E402

_spec = importlib.util.spec_from_file_location("new_card", os.path.join(os.path.dirname(__file__), "new-card.py"))
new_card = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(new_card)
ROOT = new_card.ROOT  # companion tools may live in the unmerged lane; sources belong to the caller's Git worktree
CARDS = os.path.join(ROOT, "crates", "cantrip-cards", "src", "cards")

KINDS = r"\b(?:CounterKind|Subtype|Color|CardType|Supertype|Keyword|Mana|ZoneKind|Step|Phase)::\w+"
MIN_STATEMENTS = 2


def functions(path):
    """(name, normalized body, statements) for each `fn` in a file, found by brace matching."""
    with open(path, encoding="utf-8") as source:
        text = source.read()
    text = re.sub(r"//[^\n]*", "", text)
    out = []
    for m in re.finditer(r"\bfn\s+(\w+)[^{;]*\{", text):
        depth, i = 1, m.end()
        while i < len(text) and depth:
            depth += {"{": 1, "}": -1}.get(text[i], 0)
            i += 1
        body = text[m.end():i - 1]
        norm = re.sub(r"\s+", " ", re.sub(r"\b\d+\b", "N", re.sub(KINDS, "KIND", body))).strip()
        statements = norm.count(";") + norm.count("=>") + 1
        if re.fullmatch(r"(?:return )?(?:cantrip_mechanics::\w+::|effects::)\w+\([^{};]*\)\??;?", norm):
            statements = 1  # already composes a shared recipe; no second copy to extract
        out.append((m.group(1), norm, statements))
    return out


def main():
    names = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not names:
        sys.exit(__doc__)
    index = {}
    for file in sorted(os.listdir(CARDS)):
        if file.endswith(".rs") and file != "mod.rs":
            for fname, norm, statements in functions(os.path.join(CARDS, file)):
                index.setdefault(norm, []).append((file, fname, statements))
    found = False
    for name in names:
        path = os.path.join(CARDS, new_card.file_stem(name) + ".rs")
        if not os.path.exists(path):
            print(f"{name}: no card file")
            continue
        mine = os.path.basename(path)
        for fname, norm, statements in functions(path):
            if statements < MIN_STATEMENTS:
                continue
            others = [(f, n) for f, n, _ in index.get(norm, []) if f != mine]
            if others:
                found = True
                print(f"{mine} fn {fname}: the same body as")
                for f, n in others:
                    print(f"    {f} fn {n}")
    if not found:
        print("no function body is shared with another card file")
    return 0


if __name__ == "__main__":
    sys.exit(main())
