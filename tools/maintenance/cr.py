"""The Comprehensive Rules in docs/rules (the newest MagicCompRules file there), by rule number or by keyword.

    python tools/maintenance/cr.py 702.21          # a rule and every subrule under it
    python tools/maintenance/cr.py 601.2c          # one subrule
    python tools/maintenance/cr.py trample "scry"  # a keyword (702) or keyword action (701) by name

    import cr
    cr.rule("702.21")            # the text, or None
    cr.section("Ward")           # ("702.21", "Ward") or None: the keyword or keyword action of that name
    cr.terms_in(text)            # the 701 keyword actions a text uses, as (number, title)
"""

import functools
import glob
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))


@functools.lru_cache(maxsize=None)
def _rules():
    """(the rule lines, {number: index of its line}) of the body of the newest CR file (after its contents)."""
    files = sorted(glob.glob(os.path.join(ROOT, "docs", "rules", "MagicCompRules*.txt")))
    if not files:
        return [], {}
    lines = open(files[-1], encoding="utf-8-sig").read().splitlines()
    where = {}
    for i, line in enumerate(lines):
        m = re.match(r"^(\d{3}(?:\.\d+[a-z]?)?)\.? ", line)
        if m:
            where[m.group(1)] = i  # the body comes after the contents, so the last line of a number wins
    return lines, where


def rule(number):
    """A rule and its subrules (702.21 -> 702.21, 702.21a, 702.21b ...), or one subrule (601.2c), or None."""
    lines, where = _rules()
    number = number.rstrip(".")
    if number not in where:
        return None
    out = []
    pattern = re.compile(r"^" + re.escape(number) + r"(?:[a-z]|\.\d+[a-z]?)?\.? ")
    for line in lines[where[number]:]:
        if not line.strip():
            continue
        if not pattern.match(line):
            break
        out.append(line)
    return "\n".join(out)


@functools.lru_cache(maxsize=None)
def _titles():
    """{lowercase title: (number, title)} of the keyword actions (701) and keyword abilities (702)."""
    lines, where = _rules()
    out = {}
    for number, i in where.items():
        if re.fullmatch(r"70[12]\.\d+", number):
            title = lines[i].split(" ", 1)[1].strip()
            if len(title) < 50:
                out[title.lower()] = (number, title)
    return out


def section(name):
    """The 701 or 702 rule a keyword or keyword action is, by its name ("Cumulative Upkeep", "scry"), or None."""
    return _titles().get(name.strip().lower())


def terms_in(text):
    """The keyword actions (701) a text uses, as (number, title), in rule order: "scries", "sacrificed" and
    "investigate" all count; 701.1, the section's introduction, never does."""
    found = []
    low = text.lower()
    # Longest title first, its words then blanked out, so "manifest dread" is not also "manifest". No "-ing": Oracle
    # text does an action, and "an enduring story" is not endure.
    for key, (number, title) in sorted(_titles().items(), key=lambda kv: -len(kv[0])):
        if not number.startswith("701.") or number == "701.1":
            continue
        if key == "counter":  # the verb, not "+1/+1 counter" or "counters on it"
            pattern = r"\bcounter(?:s|ed)? (?:target|that|it|the|all|each|up to)\b|\bcountered\b"
        else:
            stem = re.escape(key)
            stem = stem[:-1] + "(?:y|ie)" if key.endswith("y") else stem.rstrip("e")
            pattern = r"\b" + stem + r"(?:e|es|s|d|ed|ies)?\b"
        if re.search(pattern, low):
            found.append((number, title))
            low = re.sub(pattern, " ", low)
    return sorted(set(found), key=lambda nt: tuple(int(x) for x in nt[0].split(".")))


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for arg in sys.argv[1:]:
        number = arg if re.match(r"^\d{3}", arg) else (section(arg) or (None,))[0]
        text = rule(number) if number else None
        print(text if text else f"no rule {arg!r}")
        print()


if __name__ == "__main__":
    main()
