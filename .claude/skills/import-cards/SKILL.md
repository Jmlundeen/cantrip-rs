---
name: import-cards
description: Implement every card of a list that the engine doesn't have yet. The list can come from any source - a CubeCobra cube (kept in docs/cube-lists and synced), a decklist (a file, an MTGO or Arena export, a deck in decks/ - main deck and sideboard), a set by code or name, a Scryfall search, a local text file, or names pasted in. Its missing cards go to parallel card agents (the add-cards skill). Takes the source and an agent count, and asks for either one that is missing. Use when asked to import, sync, update or refresh a cube, to add the cards of a deck, a set or a list, or to make a list of cards playable.
---

# Importing a list of cards

One list per run, in three steps: **resolve** the source to card names, **keep** the list when it is a cube, and
**implement** every name on it that no card file defines. Every step covers that list and nothing else. This skill
doesn't implement cards itself. Step 3 runs the `add-cards` skill, which runs its agents. This skill only adds what
comes before it and the report after it.

## Arguments

- **Source:** exactly one of these.
  - **A CubeCobra cube:** a list already kept from CubeCobra (a name in `docs/cube-lists/cubecobra.json`, such as
    `LSVCube`), or any cube on CubeCobra. Name it by its short id (the case-sensitive name in
    `cubecobra.com/cube/list/<id>`), its long id or its whole address. An optional list name for a new cube may follow
    (`as <List>`).
  - **A decklist:** a file (a deck in `decks/`, an MTGO or Arena export), or one pasted into the conversation.
  - **A set:** its code or its name, as Scryfall has it (`mh3`, `Modern Horizons 3`).
  - **A Scryfall search:** any query (`o:earthbend`, `t:dragon cmc<=3 is:commander`).
  - **A local text file:** names one a line, with counts or not.
  - **Names given in the request**, pasted or typed.
- **Agents:** how many card agents run at once. This is concurrency, not a card limit: the run implements every
  missing card on the list.

They are given as `<source> <agents>` (for example `LSVCube 3`, `set:mh3 4`) or in words. If either is missing, ask
with one `AskUserQuestion` that covers both. For the source, offer the lists in `cubecobra.json`, and the user can give
anything else under "Other". For agents, offer 2, 3, 4 and 6, with **4** recommended. If the user names more than one
source, ask which one this run is for. The others wait for runs of their own.

## Names, the way the card files spell them

The card files use one spelling for each card: the front face of a double-faced, adventure, flip or meld card, and
`A // B` for a split card. **A Through the Omenpaths reprint (set `om1`) is never used for anything.** Its new name is
mapped through the oracle id to the original printing's name (`docs/cube-lists/printed-names.json` keeps the answers),
so "Makdee and Itla, Skysnarers" is listed, built and counted as Spider-Woman, Stunning Savior. Both tools below write
names this way. Never retype a name by hand.

## 0. Before anything

**The main checkout must have no tracked changes:** `git status --porcelain --untracked-files=no` prints nothing. The
card agents branch from `HEAD` and land on `main`, so uncommitted work would be in their way. If there are changes, ask
whether to commit them first or stop.

## 1. Resolve the source

### A CubeCobra cube: sync it, and keep it

A cube is kept as a list in `docs/cube-lists/`, where the draft and the events read it. `docs/cube-lists/cubecobra.json`
maps each list name to the id of its CubeCobra cube: `{"LSVCube": "LSVCube"}` makes `docs/cube-lists/LSVCube.txt` the
mainboard of cubecobra.com/cube/list/LSVCube. A cube that isn't kept yet is fetched whole into a new list, named after
its short id unless `--as` gives another name, and the sync adds that mapping, so later runs only need the name.

A list that is not in the mapping file is kept by hand, and the sync refuses to write a cube over it. Don't work around
that by guessing a mapping because two names look alike. `LSVPower` on CubeCobra is someone else's cube, not the source
of `PowerLSV`. If the refusal names a hand-kept list, ask the user either for another name (`--as`) or to confirm that
the cube really is that list's source. If they confirm, add the mapping to `cubecobra.json` yourself and sync again.

1. `python tools/maintenance/cubecobra-sync.py <cube> --check [--as <List>]`. It prints the cube's added (`+`) and
   removed (`-`) names, or every name for a new list, and writes nothing. Its last two lines are `list: <List>`, the
   name of this run's list from now on, and `changed: yes` or `changed: no`. If it fails, stop and report the message:
   a cube that isn't found (short ids are case-sensitive, so ask for the exact address), a hand-kept list in the way
   (see above), or no network. Never write a list by hand to work around a failure.
2. If it says `changed: yes`, run the same command without `--check`. The script rewrites the list whole from the
   cube's mainboard, in CubeCobra's order. A card that leaves the list keeps its card file.
3. **Commit the list.** A list that changes leaves its pick order stale, and a new list has none, and
   `CubePickOrderTests` checks that `<List>.picks.json` ranks exactly the cards on the list.
   - `python tools/maintenance/pick-order.py <List>`. Cards with no rating yet are looked up on CubeCobra one at a
     time, about half a second per new card. A card it reports as having no rating is ranked last, which is fine.
   - `tools\test.cmd /noweb`, redirected to a file, then read its last line. If it is red because of the list, the
     cause is a test that names a card or counts the cube. Report that to the user rather than editing the list to make
     it pass.
   - Commit the list, its pick order and `cubecobra.json` (when it changed) in one commit, with a message naming the
     cube and the counts (`LSVCube from CubeCobra: 12 cards in, 15 out`, `LSVPower from CubeCobra: a new cube, 544
     cards`), then `git push`. The card agents branch from `HEAD`, so the list has to be committed before any of them
     starts.
   - On `changed: no`, say so and go on. A card can be on the list and still have no card file, for example after an
     earlier run gave it up.

In the steps below, the list is `--list <List>`.

### Anything else: one list of names

`tools/maintenance/card-list.py` turns any other source into the names the card files use, one per line, checked
against Scryfall:

```
python tools/maintenance/card-list.py decks/rw-aggro.txt --out <scratchpad>/<name>.txt
python tools/maintenance/card-list.py - --out <scratchpad>/<name>.txt          (a pasted list, on standard input)
python tools/maintenance/card-list.py "set:Modern Horizons 3" --out <scratchpad>/<name>.txt
python tools/maintenance/card-list.py "query:o:earthbend" --out <scratchpad>/<name>.txt
```

- A decklist may use `4 Name`, `4x Name`, `Name (SET) 123` or `SB: Name` lines, and section headers.
- **The sideboard is part of the deck.** A deck is complete only when every card in its main deck *and* its sideboard
  has a card file, so the sideboard is always imported, whatever form it takes: a `Sideboard` header, `SB:` lines, or
  the cards after the blank line of an MTGO or Arena export. The commander and companion are included too; a
  maybeboard is not. Never pass `--main-only` for an import, and never report a deck done, or skip its sideboard cards
  as optional, while a sideboard card is missing.
- A set is every card Scryfall lists in it: basics, promos and reprints included, tokens and art cards left out.
  Cards that already have card files are left out at step 2 anyway.
- Write the list to the scratchpad. It belongs to this run, not to the repository. A decklist goes into `decks/` only
  if the user asks for the deck to be kept (decks are data files, one per deck, with a comment header like the others).
- **Unknown names stop the run.** The script reports each name Scryfall doesn't know on standard error and exits with
  status 1. Show them to the user and ask for the right spelling, or whether to drop them. Never guess. A name that
  only a fuzzy match found is reported as `matched 'X' as 'Y'`. Show those too, so a wrong guess gets caught.

In the steps below, the list is `--file <scratchpad>/<name>.txt`.

## 2. Find what is missing

`python tools/maintenance/missing-cards.py <the list>` prints the names on the list that no card file defines, and
`--count` gives the total. Show the user the list. For a cube it includes cards that were missing before this sync, and
those are part of this run too. Cards on other lists are not part of it. If none are missing, skip to the report.

## 3. Implement them

Invoke the `add-cards` skill with the missing names, `<agents>` agents at a time, and follow it **completely**, as its
coordinator (`.claude/skills/add-cards/coordinator.md`): dispatch, gaps, review, landing and its report. A card it skips for a core gap is listed in
the final report with the gap. The names go to the agents in the order `missing-cards.py` prints them. Never sort them
by difficulty.

## Report

Once every agent has finished and the cards have landed, end with what `add-cards` reports, preceded by the source. For
a cube, give its list, its added and removed names (or the count, for a new list), and the commit that carried them. For
anything else, give the source, how many names it had, and how many were missing.
