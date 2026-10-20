# Claims in full

Every card that is worked on is an issue on GitHub, claimed by whoever works on it, so two people never build the same
card. `python tools/maintenance/card-issues.py` does all of it (`--help` for the rest); `gh` must be signed in.

1. **Which cards.** If the request names cards, those are the cards. Otherwise they come from a format's list of
   cards that decks actually play (`docs/formats/<format>.json`, best-played first, made from MTGGoldfish and MTGTop8
   by `format-scrape.py`), and the format is the contributor's to choose: **ask which format** unless they said (the
   lists are the files in `docs/formats/`). `card-issues.py next <format>` prints the next cards that are not
   implemented and that nobody holds - a claim older than 24 hours with no pull request commit or `card/<slug>` push
   since counts as nobody's. Take them in that order and never skip one for being hard; a card the engine cannot yet
   do is held while its gap is built in the core, not passed over silently.
2. **Claim one card at a time, before any work on it:** `card-issues.py claim --format <format>` takes the next free
   card, or `claim "Card Name"` the one named. The next card is claimed only when this one has landed or been
   released - or is waiting on a core gap, which stays claimed while the next card is worked (never more than those
   two) - because a pile of claims keeps others off cards nobody is working on. `claim` opens the card's issue first
   when it has none (a card named outside every format list gets one too), claims it with a comment and prints `#N`.
   Exit status 3 means the card is held by someone else (or they claimed it a moment earlier), or nothing is free;
   the message says which. Never work on a card whose live claim is someone else's.
3. **A card that already exists is validated, not rebuilt** (`SKILL.md`, "Which card, and claiming it").
4. **Keep the claim alive** - it lasts 24 hours from the last sign of work. Open a (draft) pull request whose
   description says `Closes #N` as soon as there is a commit, or push work in progress to a branch named
   `card/<card-slug>`; every commit pushed there renews the claim. A card waiting on a core gap keeps its claim, and the
   claim is never renewed by hand: 24 hours is far more than any card takes, core gaps and all, so a claim that lapses
   means the work stopped. A card that is abandoned - nobody will build its gap, or the work stops - is released
   (`card-issues.py release "Card Name"`) with its gap reported on the issue.
5. **Landing it closes the issue:** the commit message (and a pull request description) ends with `Closes #N`.

## The progress log

Every card's issue carries a log of where its work stands, so how long cards take is measured, not guessed: each point
is one fixed comment, `python tools/maintenance/card-issues.py log <N> <event> [input]`, and the comment's time is the
event's. Nothing is written by hand - an event takes at most one input.

| Event | Who logs it | When |
| --- | --- | --- |
| (the claim) | the author | `claim` - the start |
| `committed --commit <sha>` | the author | the card is committed and waits for review |
| `review-started` | the reviewer | review begins |
| `review-blocked --findings <n>` | the reviewer | review ends with blocking findings |
| `fixes-committed --commit <sha>` | the author | the fixes are committed |
| `review-passed` | the reviewer (or the coordinator, verifying the fixes) | review ends, nothing blocking |
| `blocked --rule <CR>` | the author | the card waits on an engine change |
| `unblocked` | whoever resumes the card | the engine change is ready |
| `landed --commit <sha>` | whoever lands it | it is on `main` |

`card-issues.py times [--days 30]` reads the logs back: per card, the minutes from claim to commit (implementing),
commit to review passed (review and fixes), passed to landed, the whole, and the time it waited on the engine, with
medians - the numbers a change to the process is judged by.
