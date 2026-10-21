---
name: fetch-card-art
description: Download card art for cards that have none, from mtgpics.com (then Scryfall art crops for what it lacks) - the art alone (not the card image), the original non-promo printing, at the best resolution available - into content/mtg/images under the client's slug file name. Use when asked to scrape, fetch or fill in missing card art, or after import-cards ends with a list of faces that still need art.
---

# Fetching card art from mtgpics

Art is a file per face in `content/<dataset>/images/<slug>.jpg` (`CardArt.Slug`: lower case, hyphens, no apostrophes -
`Umezawa's Jitte` is `umezawas-jitte.jpg`). The folder is gitignored; art never goes into git.

## Steps

1. **List what is missing.** `python tools/maintenance/missing-art.py` (faces, one a line) and `--tokens`: every card
   with a card file under `content/*/cards/`. `--list <Cube>` or `--file <path>` narrows to one list's cards.
2. **Drop what has no art to find.**
   - Tokens: not covered here. mtgpics rarely has them; ask before spending time.
3. **Check for misnamed files first.** `ls content/mtg/images | grep -v '^[a-z0-9-]*\.[a-z]*$'` - art the user dropped
   in under a display name (`Appa, Steadfast Guardian.jpg`) is already art: rename it to its slug instead of fetching.
4. **Fetch into a staging folder** (a scratch directory, not the images folder):
   `python tools/maintenance/mtgpics-art/fetch_art.py <stage> < names.txt`. About 2 s a card, so run it in the background.
   Needs Pillow. One line a card: `OK`, `OK SCRYFALL`, `OK PROMO-ONLY`, `NOART`, `ERR`.
5. **Check, then install.** List staged sizes; look at anything odd by reading the image. Install with
   `python tools/maintenance/mtgpics-art/fetch_art.py content/mtg/images --install` (the same names), or copy the staged files; a
   file already there is replaced only by a larger picture. Re-run `missing-art.py` - what it still lists is the report.
6. **Report** what was not found and why, plus the cards that came back small or promo-only. Don't commit.

## Scryfall fallback (user-approved 2026-10-16)

mtgpics has no art-only picture for most recent sets (Torch the Tower, the Marvel and Hobbit cards). `NOART` faces go to
`python tools/maintenance/scryfall-art/fetch_art.py <stage> < noart.txt`: the earliest regular printing's `art_crop`, ~600x450 -
low resolution, but better than none. Run mtgpics first; Scryfall only for what it lacks. It never replaces a file.

## Why it chooses as it does (the user's rules)

- **Art only**: `pics/art/<set>/<n>.jpg`, never `pics/big/` (the card image).
- **Original art, never promos**: Secret Lair, box toppers, Special Guests, anniversary and similar sets are skipped;
  the earliest regular release wins. A first version picked "largest pixels" and returned square Secret Lair art.
  A larger copy of the *same* picture (a reprint, a better scan) may replace it; a different illustration never does.
- **Scryfall names the original, mtgpics supplies the pixels.** mtgpics files a set's showcase, borderless and Japanese
  variants under the set's own code (`fin/379` beside `fin/089`) and misses some first sets (Alpha), so its dates
  picked showcase art for about one card in six. The fetcher compares every mtgpics picture with Scryfall's crop of the
  earliest regular printing and keeps the largest that matches; with none, a modern card gets Scryfall's crop
  (`OK SCRYFALL`). Old-frame scans differ too much to compare and fall back to mtgpics' earliest printing.
- **Portrait art is fine** (planeswalkers, full-art frames): the client crops.
- Old originals are small (400-700 px). That is the cost of favouring the original; offer a larger regular reprint
  if the user wants one, do not pick it unasked.

## Known gaps

- `NOART` means mtgpics lists no art for the card (Channel, Uldaros Theorix, Skycoach Conductor were such). The page
  lists distinct arts of a card name, and some old printings (Eternal Witness's Judgment art) are not on it.
- mtgpics answers 500 now and then; the script retries. If a whole run errors, wait and rerun the failures only.
- Be polite to the site: the script sleeps between requests; don't parallelise it.
- The layout was found by hand (no API). If it stops working, the doc comment at the top of `fetch_art.py` says
  what each URL is expected to return.
