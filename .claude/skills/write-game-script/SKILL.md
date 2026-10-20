---
name: write-game-script
description: Write or fix a scripted game test (*.cantrip) for the Cantrip rules engine — two ordered decklists, a sparse action list per bot, and expected final state. Use when adding or changing a card, mechanic, prompt, or rules interaction and it needs an end-to-end test, or when a .cantrip script is failing.
---

# Writing game scripts

Scripted games are the way to test cards and interactions end to end. The cards are really played, so triggers,
replacement effects and state-based actions all happen. A position is forced by the script, never reached by
sweeping seeds (CLAUDE.md).

- Scripts: `scenarios/*.cantrip`. Each file is its own test case. `scenarios/README.md` has the full syntax and the
  list of dump keys. Read it first.
- Harness: `crates/cantrip-script` — `src/parse.rs` (the parser), `src/run.rs` (the runner: matching lines to the
  engine's questions, the defaults, the failure report), `src/dump.rs` (the state dump `[Expect]` is checked against).
- Card names are the `"name"` of a card file in `content/*/cards/*.json`, matched case-insensitively. A scenario runs
  only when every card in both decks exists in the engine (has a file in `crates/cantrip-cards/src/cards/`, or needs
  none); otherwise `corpus` lists it with the cards it still needs.

## Workflow

1. **Plan the turns on paper.** Bot1 plays on odd turns and Bot2 on even turns, until an extra turn (Time Walk)
   shifts them. Bot1 skips its turn-1 draw. One land per turn, so an N-mana spell is castable on the controller's Nth
   turn at the earliest.
2. **Build the decks top first.** The first 7 lines are the opening hand, and the next lines are the draws in order.
   Give each deck at least 7 cards plus one per draw step the script reaches, and pad generously (`Island x10`). A
   player who draws from an empty library loses and fails the script. For the other cards on the board - a blocker
   of toughness 4, a creature with flying and nothing else, a cheap colorless artifact - take the plainest implemented
   card: `python tools\maintenance\find-cards.py --type creature --toughness 4` (also `--power`, `--mv`, `--color`,
   `--keyword`, `--text`, `--vanilla`) lists them least rules text first. Free mana for a test: the Moxen and Black
   Lotus, cast for nothing.
3. **Write the action list.** Include only what the test needs: land plays, mana taps, casts and choices. Everything
   else passes by default.
4. **Run it:**
   ```
   cargo run --profile cards -p cantrip-script --bin corpus -- --card "Card Name"
   ```
   That runs every scenario whose decks hold the card and prints the full report of any that fail. When it passes,
   the scenario is already a test: `cargo test` makes one test per file in `scenarios/` (nothing to list or
   regenerate), and `corpus` with no arguments runs them all. Run `tools\test.cmd` before finishing.
5. **Read the failure report.** It gives the reason, the outstanding question with every option it offered (as the
   script would name them), a trace of the last lines with the turn each fired on and every default answer given, and
   the full state dump.
6. **Write the expectations from the dump.** To see the dump of a passing script, add a bogus expectation such as
   `Dump = ?`: the mismatch fails the script and prints the dump, with the keys nearest the bogus one. Copy only the
   keys that prove the behaviour, then remove the bogus line.
7. **Check that the script can fail.** Temporarily break an expectation, or the rule under test, and confirm the
   script goes red. A script that passes vacuously is worse than none. Two places a card's test most often passes
   when it should not:
   - **A numeric limit** ("mana value 2 or less", "power 4 or greater", "up to three"): put an object exactly at the
     limit on the side that qualifies and one just past it on the side that does not, so `<` for `<=` fails.
   - **Whose things it reads** ("each", "all", "target", "you control", "an opponent controls"): give both players
     something it could apply to, so a filter that reads only one side (or the wrong one) fails.
8. **If the change added a shared mechanic or a core feature, write a second script for a second composition** — a
   different card, or a different arrangement of the same one, going through the same code. One caller never
   demonstrates that a capability is general.

## How lines are matched (the gotchas)

- **One cursor for both bots.** A line fires the first time its bot is asked something that offers it. While the
  cursor is on Bot2's line, Bot1 only gives default answers, even if Bot1 could do something useful.
- **As soon as possible means early.**
  - A mana ability is offered at every priority, so `Island.Activate` fires in upkeep if nothing earlier holds it back.
    Mana empties between steps, so the later cast then has nothing to pay with. Put the tap right after something that
    only happens in the main phase, such as the land play, or put a wait first (`[Bot1.Main1]`).
  - To act during a resolution rather than in response, insert `BotN.Pass`. For example, tap lands during Mana Leak's
    unless-cost instead of while it is on the stack.
- **Waits match immediately if already true.** `[Bot1.Main1]` written during Bot1's current main phase is consumed on
  the spot. Add the turn: `[T7 Bot1.Main1]`.
- **The caster keeps priority after casting.** The opponent's response line fires once the caster passes, which
  happens automatically when the next line isn't the caster's.
- **Building a stack.** Every spell waits on the stack while its caster holds priority, permanent spells too: a Mox or
  a Black Lotus is not on the battlefield until both players pass. A line that is not offered yet (tapping that Lotus)
  is reached by default passes, which resolve the stack until it is; but a line that is offered at once - another
  instant, a `? Stack.Count` check, a counterspell's `Target` - sees everything still there, the Moxen included. So
  put `BotN.Pass` after a cast that must have resolved before the next line is answered. Only an instant, or a
  spell with flash, can be cast onto a non-empty stack: a sorcery-speed line waits while everything under it
  resolves. To set up several spells for one response (a counterspell choosing among targets), resolve the mana
  first, cast the sorcery-speed spell, then the instants, then the response. A line left waiting for something that
  already happened shows in the trace as "the game ended before line N".
- **Targets gate matching.** A cast waits until every named target is a legal candidate. A spell with no legal
  target is not offered at all, so a line like `Counterspell.Cast(Bot2)` never matches and fails after 4 turns.
- **Paying.**
  - Tap before casting, or follow the cast with `Activate` lines, which answer the payment prompt.
  - `X: n` is handled automatically.
  - If a spell is left owing mana, the script fails; it is never quietly cancelled.
- **Defaults.** When the cursor's line doesn't apply, bots:
  - pass priority
  - attack and block with nothing
  - decline unless-costs
  - choose the first eligible cards
  - find nothing when searching
  - leave looked-at cards in place
  - discard the newest cards to hand size

  The defaults are `default` in `crates/cantrip-script/src/run.rs`, and each one given appears in the trace as
  `(default)`. Bots with nothing to do draw and discard every turn, which changes `Hand.Count` and `Graveyard`.
- **Order-sensitive choices.** For Brainstorm-style `Choose`, the last card chosen ends up on top. `Arrange` lists the
  top cards top first, then `|`, then the bottom cards (for surveil, the cards going to the graveyard).
  An "up to" choice (Frantic Search's lands to untap) takes fewer, or none.
- **Ambiguity.**
  - Objects on the stack are named `Stack.<Name>` (e.g. a target `Stack.Lightning Bolt`), as the failure report shows.
  - Same-named cards in one zone are interchangeable, and the first one offered is used.
  - A target name matching permanents of both players must be qualified (`Bot2.Battlefield.Grizzly Bears`).
  - In `[Expect]`, a name that appears more than once in a zone needs an index: `Bot1.Battlefield.Island#1.Tapped`.
    So do two same-named cards chosen together in one answer: `Bot1.Target(Ornithopter#1; Ornithopter#2)`.
  - A nameless object (a face-down permanent) is `#1`: `Bot1.Battlefield.#1.Activate`.
  - A permanent with several abilities needs `Ability: <part of its text>`, such as `Ability: {U}` for Tundra.
  - A modal double-faced card with two land faces is played by face: `Play(Ability: #1)` the front, `Play(Ability: back
    face)` the back (its option reads "as its back face").
  - A cast for an alternative cost names it the same way (`Cast(Duress; Ability: Return)` for Daze).
  - Mana of any one color needs `Color:` (`Color: C` for colorless).
- **Fixed dice.** A `[Dice]` section fixes the faces rolled, in order (README).
- **Starting conditions, not board state.** A scenario may set its starting conditions (life with `[Life]`, dice with
  `[Dice]`), but every counter and every piece of board state comes from game actions played in the script: poison
  counters from infect or toxic damage, never written in.
- **Cards outside the game.** A `[Bot1 Sideboard]` section, written as a deck is, is what Bot1 owns outside the game (CR
  400.11a), for a card that brings one in (North Wind Avatar); `[Basic Lands Outside]` makes it a Limited game whose
  sideboards hold any number of the basic lands it lists. `Bot1.Sideboard` is the dump's key (README).
- **Where scripts end.** A script stops when every line has fired and the stack is empty. It does not play on to
  combat damage or the next turn unless a trailing wait (`[Bot1.Main2]`) says so.
- **Check timing.** `? Key = Value` in the action list is checked at the next question after the previous line.
  After a cast the spell is still on the stack. After a choice made during resolution, the spell has usually
  finished. To check a permanent that a cast puts onto the battlefield, put the check after a later line or a wait.
- **Proving something is not allowed.** `! Bot1.Graveyard.Card.Cast` fails if that player's next priority offers the
  action (a refused `Cast` without `Ability:` covers every way to cast it; unaffordable still counts as offered).
  `! Bot2.Block(Blocker > Attacker)` and `! Bot1.Attack(Attacker > Defender)` fail if that player's next declaration
  allows the pair. `! Bot1.Search(Plains)` fails if their next library search may find a Plains. A refusal lands on
  the *next* question, so place it where the action would otherwise be legal (main phase, empty stack), and give a
  targeted spell a target. Pair each refusal with a positive control — the same action taken where the rule under
  test does not apply — or it can pass for the wrong reason.
- **Refusals that name targets or questions.** `! Bot2.Hand.Lightning Bolt.Cast(Bot1.Battlefield.Figure of Fable)` refuses only the
  option announceable with that target (how a hexproof or protection limit is proved). A bare `! Bot1.Choose` (also `Decline`,
  `Colors`, `Name`, `Arrange`, `ChoosePlayer`, `Modes`, `Target`) fails if that question is asked before the player's next priority.
- **Zone lists in `[Expect]` have an order.** `Graveyard` and `Library` are top first: the card put into a graveyard
  last comes first, so a spell that resolved and went to the graveyard is listed before what it destroyed, and two
  cards destroyed together keep the order they arrived in, newest first. `Hand`, `Battlefield` and the other lists are
  sorted by name. When the order is not what the test is about, assert `.Count` and the per-card keys instead.
- **Asserting a key is missing.** `Bot2.Battlefield.Bat.Keywords = (absent)`; it fails unless the dump has some
  other key on that object, so a typo cannot pass. A counter kind at zero is absent from the dump.

## When the engine is wrong

Assert what the rules require. If the engine or a card does something else and the fix is not part of this change,
commit the scenario anyway with `# Known bug: <what, and the rule>` as its first line (`scenarios/README.md`, "Known
bugs"): it is expected to fail, and passing reports it fixed. Never delete or weaken a scenario because the engine
gets it wrong.

## When the harness can't express it

Extend it; never reach a position by seed.

- **A new question** (`RequestKind`) gets a default in `run.rs` `default`, so a script runs without a line for it,
  and a verb in `parse.rs` (`ScriptVerb`) plus its matching in `run.rs` only when a script needs to answer it
  deliberately.
- **A new kind of option** (an `ActionOption` slot, modes, several targets) is matched in `run.rs` `matches`.
- **New state worth asserting**, such as counters, attachments or emblems, becomes keys in `dump.rs`. Keep keys
  stable and `;`-separated, and mark unordered lists as such.
- **Update the docs and tests.** Keep `scenarios/README.md` in step, and add a parser test in `parse.rs` for each new
  piece of syntax.

Name scripts as the behaviour they prove, e.g. `mana-leak-paid-by-tapping-while-it-resolves.cantrip`. Start each file
with a `#` comment saying what it shows and why the actions are arranged that way.

## Stopping at a question

A script can stop with a question outstanding instead of answering it:

```
Bot2.Block(Vampire Nighthawk > Hill Giant; Serra Angel > Hill Giant)
[Handover Bot1.AssignDamage]
```

Named by the verb that would have answered it. Its `[Expect]` block then describes the position mid-game
(`Step = Damage`, `Combat.Blocks = ...`, damage still `0`), and `cantrip-bench` measures answering such questions.
Name the question: a bare `[Handover]` takes the next question of any kind, which is usually one or two early.
Until the named question is asked the seats answer by default, and a handover that never arrives fails after four
turns like any other line.

Serving a handed-over position to the web client, to look at or play on from, went with the C# engine and is not
rebuilt yet. Until it is, a client screen is tested in `dotnet/tests/Cantrip.Client.Web.Tests` (`TablePage` finds
cards and controls by name on `globalThis.cantripTest`; `docs/web-client.md`).
