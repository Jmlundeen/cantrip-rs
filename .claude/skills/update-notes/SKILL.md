---
name: update-notes
description: Write the player-facing update notes for Cantrip - what went into the public repository (jschibel/cantrip-rs) since the last notes were sent - as three Discord bullet lists, new cards, new core and shared mechanics, and bug fixes, then record the newest commit as reported. Use when asked for update notes, patch notes, release notes or a changelog to post.
---

# Update notes

The notes cover the public repository, https://github.com/jschibel/cantrip-rs, from the last commit reported up to its
current `main`. That last commit is kept in `.claude/skills/update-notes/last-reported-commit.txt` (gitignored; one
line, a full sha).

## 1. Gather

```
python .claude/skills/update-notes/gather.py
```

It fetches the public `main` and prints the range (`FROM`, `TO`), the cards added, files added to
`cantrip-mechanics`, known issues whose entries were deleted, and every commit's subject and body, oldest first. If it
says there is nothing new, tell the user so and stop: no notes, nothing saved.

## 2. Write the three lists

Plain Discord markdown: a bold heading per list, then one item per line starting with `- `. Nothing else - no
preamble, no commit hashes, no file paths, no rule numbers, no internal names (crates, `KI-` ids, protocol numbers,
function names). Write for players: say what they can now play or what now works right.

**New cards** - exactly the names under NEW CARDS, alphabetical, one per line (`- Card Name`; a double-faced card as
printed, `Front // Back`). Do not add cards from commit subjects: a `Card:` commit can be scenarios or a fix to a card
already out.

**New mechanics** - core rules and shared mechanics a player would notice: keyword abilities and keyword actions
(`Mechanics:` commits, new `cantrip-mechanics` files), and rules the engine now handles (`Engine:` commits that add
something rather than fix it - a new kind of effect, cost, designation, zone rule). One line each, named the way the
rules name it (`- Earthbend`, `- Loyalty abilities at instant speed`). Leave out refactors, card-syntax and harness
work, protocol bumps, tooling, docs and scenarios.

**Bug fixes** - what was wrong and now works, as a player would have seen it: `Fix:` commits, `Engine:`/`Client:`
commits that correct behaviour, deleted known-issue entries, and `Card:` commits that correct a card already released
(current Oracle text, a wrong interaction). One line each, short (`- Daydream returns the creature under its owner's
control`). Merge commits that fix the same thing. A fix to something added in this same range is not a fix to report.

A list with nothing in it says `- None` under its heading. When a commit's subject is unclear, read the commit
(`git show <sha>`) rather than guess.

Give the user the notes in one fenced code block, ready to paste:

```
**New cards**
- ...

**New mechanics**
- ...

**Bug fixes**
- ...
```

## 3. Record the commit

Once the notes are written, save the `TO` sha from step 1 (not whatever `main` is by now):

```
python .claude/skills/update-notes/gather.py --save <TO sha>
```

Tell the user the range covered, `FROM` to `TO`, by short sha.
