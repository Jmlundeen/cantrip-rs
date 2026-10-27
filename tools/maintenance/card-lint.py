"""The add-cards skill's "never" list, checked mechanically over a card's file before anyone reviews it.

    python tools/maintenance/card-lint.py "Card Name" ["Other Card" ...]
    python tools/maintenance/card-lint.py --all          # every card file, a count per check (to tune the checks)

Each finding names its line, what the skill forbids there and what to use instead. A NEVER finding is a pattern the
skill forbids outright (an exit status of 1); a LOOK finding is one that is right only sometimes - a loop over objects
that may have to be one simultaneous event, a type word that may have to be read through the object's words - and is
answered by reading the line, not by changing it to quiet the lint. The lint is no review: it finds the mechanical
items so the review can spend its time on the rules.
"""

import importlib.util
import os
import re
import sys

import cr

sys.stdout.reconfigure(encoding="utf-8")

_spec = importlib.util.spec_from_file_location("new_card", os.path.join(os.path.dirname(__file__), "new-card.py"))
new_card = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(new_card)

ROOT, CARDS = new_card.ROOT, new_card.CARDS

NEVER, LOOK = "NEVER", "LOOK"

# (severity, pattern over one line, what the skill says). Each is an item of SKILL.md's "A card file must never" or of
# CLAUDE.md, and was checked against every card file: a NEVER pattern has no right use in a new card.
LINE_CHECKS = [
    (NEVER, r"\b(?:unwrap|expect)\(|\b(?:panic|unreachable|todo|unimplemented)!",
     "a card never crashes (CLAUDE.md): `let ... else { return Ok(()) }`, or an answer the engine already checked"),
    (NEVER, r"(?<!rest_to_)\bbottom_in_random_order\b|\bBottomInRandomOrder\b|\blibrary::random_of\b",
     "shuffling or randomising by hand: the verb that does it (`library::rest_to_bottom_in_random_order`, a shuffle)"
     " or `g.choose_at_random`"),
    (NEVER, r"\bchoose::(?:card|cards|card_from|cards_from)\(|\bsearch_library\(|\blibrary::Find\b|\blibrary::search_to_\w+\(",
     "an older choice or search kept for existing cards: a selection rule (`cantrip_mechanics::select`) asked with"
     " `choose::select`, `library::search` or `library::find_to_*`"),
    (NEVER, r"\bmove_(?:to|together(?:_revealed)?)\([^;]*Zone::(?:graveyard|hand|library)\(",
     "a zone the rules dictate built by hand: the verb (`destroy_all`, `sacrifice`, `return_to_hand`,"
     " `put_into_library`, `mill`, `discard_together`, `library::find_to_*`)"),
    (NEVER, r"\b(?:has_keyword|keywords\.(?:has|contains))\([^)]*Keyword::(?:Indestructible|Hexproof|Shroud|Ward|Protection\w*|Regenerat\w*)",
     "filtering on indestructible, hexproof, shroud, ward or protection: the verb or targeting decides that"),
    (NEVER, r"\bstatic\s+mut\b|\bAtomic[A-Z]\w*|\b(?:Ref)?Cell<|\bthread_local!",
     "a counter or flag kept for the past: ask the event history (`q.history()`)"),
    (NEVER, r"\b(?:has_lost|lost_game)\(",
     "checking whether a player has lost: `apnap()` and `opponents(p)` leave them out already"),
    (LOOK, r"\bfor\b[^{]*\bseats\(\)",
     "`seats()` is every seat, for walking zones by owner; \"each player\" is `apnap()` (or `turn_order_from(p)`)"),
    (LOOK, r"\bmove_together(?:_revealed)?\(",
     "`move_together` for a rules action that has a verb (destroy, sacrifice, exile, return, mill, discard ...)"),
    (LOOK, r"\.mana_cost\(",
     "a printed cost read where the rules may mean mana value (`Query::mana_value`: copies, tokens, X on the stack)"),
    (LOOK, r"\bask_may\(",
     "a \"you may search\" is `library::ask_may_search` (not offered when the search can't happen, CR 101.2)",
     r"\bsearch\b"),
]

# Calls that make a game change one object or player at a time: in a loop, they may be what the rules make one
# simultaneous event (`damage_together`, `destroy_all`, `exile_all`, `each_opponent_loses_life`, ...).
ONE_AT_A_TIME = r"\.(?:damage|deal_damage|destroy|exile|sacrifice|lose_life|return_to_hand|discard|tap|untap|put_onto_battlefield)\("

# A subtype or color tested as printed: `words.subtype(Subtype::Goblin)` and `words.color(Color::Red)` are the words read
# through the object (`cantrip_state::vocab`), so only a test handed the printed constant itself is found.
TYPE_USE = r"\b(?:has_subtype|has)\(\s*(?:Subtype|Color)::(\w+)"


def _words_rules():
    """The words a text-changing effect can change (CR 612.1): the colors, the creature types and the basic land types."""
    creature = cr.rule("205.3m") or ""
    listed = creature.split("All other creature types are one word long:", 1)[-1]
    types = {w.strip(" .") for w in re.split(r",|\band\b", listed) if w.strip(" .")}
    return types | {"Forest", "Island", "Mountain", "Plains", "Swamp", "White", "Blue", "Black", "Red", "Green"}


def lint_file(path, oracle_text):
    """Every finding in one card file, as (severity, line number, line, message)."""
    lines = open(path, encoding="utf-8").read().split("\n")
    text = "\n".join(lines)
    findings = []
    changeable = _words_rules()
    for i, line in enumerate(lines, 1):
        code = line.split("//", 1)[0]
        if not code.strip():
            continue
        for check in LINE_CHECKS:
            severity, pattern, what = check[:3]
            if re.search(pattern, code) and (len(check) < 4 or re.search(check[3], text)):
                findings.append((severity, i, line.strip(), what))
        # A color word or creature or basic land type its rules text uses, read without the object's words (CR 612).
        for m in re.finditer(TYPE_USE, code):
            word = m.group(1)
            if word in changeable and re.search(r"\b" + word + r"s?\b", oracle_text, re.I):
                findings.append((LOOK, i, line.strip(),
                                 f"'{word}' is a word of the card's rules text: read it through the object's words"
                                 f" (`g.words(item.object).subtype(..)`), so a text-changing effect changes it (CR 612)"))
        # A target's guard that wraps the rest of the function: when the target is illegal the spell does not resolve,
        # so the closing brace is a line no scenario runs.
        if re.search(r"^\s*if let Some\([^)]*\) = targets::object\(", code) and code.rstrip().endswith("{"):
            depth, end = 0, None
            for j in range(i - 1, len(lines)):
                depth += lines[j].count("{") - lines[j].count("}")
                if depth == 0:
                    end = j
                    break
            after = [l.strip() for l in lines[end + 1:end + 3]] if end is not None else []
            if after[:2] == ["Ok(())", "}"] and lines[end].strip() == "}":
                findings.append((LOOK, i, line.strip(),
                                 "a lone target's guard is `let Some(x) = targets::object(..) else { return Ok(()) };`:"
                                 " the `if let`'s closing brace is a line no scenario can run"))
        if re.match(r"\s*for\b", code):
            body = "\n".join(lines[i:i + 6])
            if re.search(ONE_AT_A_TIME, body.split("\n    }", 1)[0]):
                findings.append((LOOK, i, line.strip(),
                                 "a loop that changes objects or players one at a time: where the rules make it"
                                 " simultaneous, the together verb (`damage_together`, `destroy_all`, `exile_all`,"
                                 " `each_opponent_loses_life`, `each_opponent_sacrifices`)"))
    return findings


def card_path(name):
    return os.path.join(CARDS, new_card.file_stem(name) + ".rs")


def oracle_of(name):
    found = new_card.find_card(name)
    if found is None:
        return ""
    card = found[3]
    faces = [card] + [card[k] for k in ("backFace", "adventure", "omen", "aftermath") if isinstance(card.get(k), dict)]
    faces += card.get("halves") or []
    # Reminder text is no rules text (CR 207.2): the words of "Equip worthy {1} (... red and/or white.)" are the CR's.
    return re.sub(r"\([^)]*\)", "", "\n".join(f.get("oracleText", "") for f in faces if isinstance(f, dict)))


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if "--all" in sys.argv:
        counts = {}
        for file in sorted(os.listdir(CARDS)):
            if not file.endswith(".rs") or file == "mod.rs":
                continue
            for severity, _, _, what in lint_file(os.path.join(CARDS, file), ""):
                counts[(severity, what)] = counts.get((severity, what), 0) + 1
        for (severity, what), n in sorted(counts.items(), key=lambda kv: -kv[1]):
            print(f"{n:5}  {severity:5}  {what[:110]}")
        return 0
    if not args:
        sys.exit(__doc__)
    status = 0
    for name in args:
        path = card_path(name)
        if not os.path.exists(path):
            print(f"{name}: no file {os.path.relpath(path, ROOT)} (a card of characteristics alone has nothing to lint)")
            continue
        findings = lint_file(path, oracle_of(name))
        rel = os.path.relpath(path, ROOT).replace("\\", "/")
        if not findings:
            print(f"{rel}: clean")
        for severity, number, line, what in findings:
            print(f"{rel}:{number}: {severity}: {what}\n    {line}")
            if severity == NEVER:
                status = 1
    return status


if __name__ == "__main__":
    sys.exit(main())
