# Adding a card

How a card gets into the engine ([engine-design.md](engine-design.md)). A card is done when it is exact - every line
of its Oracle text, by the rules - and every scenario in `scenarios/` that uses it passes. Every card in content was
ported from the old C# engine this way; new cards follow the same loop.

## The loop

```
python tools\maintenance\new-card.py "Mind Stone"                          # read everything, write the file to start from
cargo run --profile cards -p cantrip-script --bin corpus -- --card "Mind Stone"   # its scenarios, full reports (~1 s)
cargo run --profile cards -p cantrip-script --bin corpus                         # whole corpus, one pass/fail line
set CANTRIP_VERIFY=1 & cargo test --profile cards -p cantrip-script             # every scenario, rollbacks checked
cargo run --profile cards -p cantrip-script --bin corpus -- --missing 40          # which cards block the most scenarios
```

The standard every card meets - exact to the rules, no deviation; every line of its file executed by its scenarios
(`tools\card-coverage.cmd "Name"`); its corner cases and interactions covered; an independent review - is the `add-cards` skill (`.claude/skills/add-cards/SKILL.md`).

The `cards` profile is release code with incremental compilation: an edit to one card rebuilds in about a second.
The whole corpus runs in a fraction of a second, so run it after every card rather than a part of it.

1. **Read the card**: `new-card.py` prints `content/<dataset>/cards/<name>.json` (its characteristics and Oracle
   text) and the scenarios whose decks hold it. A new card's JSON is its characteristics only; the
   build generates the Rust constant from it, so there is nothing to run. The scenarios say what has to be true, and they name the
   options of a question in the order the engine offers them ("the first one offered"), so a card asks its questions
   in the order the rules and the cards already ported do.
2. **Write the card's file** in `crates/cantrip-cards/src/cards/<card_name>.rs` (`new-card.py` writes it
   with the characteristics only). Nothing else has to change: the registry is generated from the files in that folder.
   Adding a card creates files only (its JSON, its Rust file, its scenarios) and edits none that another card's
   addition also edits.
3. **Run its scenarios** with `corpus --card`. A scenario runs only when every card in both decks exists; the rest
   are listed with what they still need. When the card's scenarios pass, they are already tests: `cargo test` makes one per file in `scenarios/`.
4. A scenario that is wrong (rules-incorrect) is not "fixed" by matching the mistake: fix the scenario, with the
   rule.

## A card's file

Characteristics are generated at build time (`crates/cantrip-oracle/build.rs` reads `content/*/cards/*.json`). Never type a
name, a cost, types or P/T. A card's file adds only behavior, with struct update syntax over its generated constant:

```rust
use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::ANY], effect }),
    ..oracle::LIGHTNING_BOLT
};
```

Definitions a card brings with it (a token, a copy's exception) go in `pub static EXTRA: &[&CardDef]` in the same
file. The generator's constant name is the card name upper-cased with every run of non-alphanumerics as `_`.

What a `CardDef` can carry (see `crates/cantrip-api/src/card.rs`):

| Field | For |
| --- | --- |
| `spell: Some(SpellDef { targets, effect })` | an instant's or sorcery's effect, and its target slots |
| `abilities: &[AbilityDef]` | activated abilities (`ActivatedDef`: cost, timing, targets, effect, or `mana` for a mana ability) and triggered ones (`TriggeredDef`: what it listens for, a match function, targets, effect, `look_back` for leaves-the-battlefield, `anywhere` for graveyard/hand triggers, `condition` for an intervening if). **Mana abilities first**: the C# engine numbers them first. |
| `ways: &[WayDef]` | alternative costs (flashback from the graveyard, Fireblast's sacrifice, Gush's return), way 1 onward; `instant_speed` offers one whenever an instant could be cast, whatever the card's type (sneak) |
| `enters: Some(fn)` | "as this enters" and "enters with": edits the `Entering` record (tapped, counters, a copy) before it is on the battlefield |
| `delayed: &[AbilityDef]` | abilities its effects set up to trigger later (`Game::delayed_trigger` with `AbilityRef::delayed(card, n)`) |
| `replacements: &[ReplacementDef]` | its replacement effects (CR 614): a `hint`, the event kind (`ZoneChange`, `Counters`, `DrawInstruction`), `applies(q, asker, &Proposal)` and `apply(q, asker, &mut Proposal)`, both pure; `own_everywhere` for "if this card would ... from anywhere". Several that apply to one event are ordered by the affected player (`Replace(...)` in a scenario). The shared "+1/+1 counters, plus one" is in `cantrip_mechanics::replacements` |
| `cast_limit: N` | "each player can't cast more than N spells each turn" (CR 101.2), while it is on the battlefield |

## Card code

An effect is `fn(&mut dyn Game, &StackItem) -> Result<(), Suspend>`. It reads the game through `Query` (`g.state()`,
`g.chars(id)`, `g.obj(id)`, `g.permanents_of(p)`...) and changes it only through `Game`'s operations (`draw`,
`damage`, `move_together`, `put_onto_battlefield`, `create_tokens`, `counter`, `unless_pays`, `add_effect`...). The
full list is `crates/cantrip-api/src/game.rs`.

**Asking a question:** `let answer = g.ask(player, RequestKind::...)?;`. The first time it is reached, the effect
stops there (`?` carries the `Suspend` out). Once the question is answered, the effect runs again from the start and
`ask` returns the answer. So:

- Card code must be deterministic: the same game and answers do the same thing. Use the game's RNG (`g.shuffle`), no
  globals, no clocks, no `HashMap` iteration.
- Everything before the question is done again on the re-run. That is correct and cheap (the engine rolls the first
  run back). Never keep state across runs anywhere but the game.
- Ask what the rules ask, when they ask it, the way the cards already ported do. A question with one possible answer
  is skipped in some places and not in others (a lone copy candidate is taken; a sacrifice of exactly as many as
  there are is taken; a discard is always asked unless the hand is empty); match the existing cards, since the
  scenarios rely on it.
- Most questions are asked through `cantrip_mechanics::choose` (`card`, `cards`, `card_from`, `cards_from`, `player`,
  `one_of`), which return the thing chosen. The core checks every answer against its question before the card sees
  it, so a card never matches on an answer of the wrong shape.

**The questions** (`RequestKind`, `crates/cantrip-state/src/decision.rs`), and the script verb that answers each:

| Question | `RequestKind` | Script verb |
| --- | --- | --- |
| choose cards | `ChooseCards { purpose, cards, eligible, count, up_to }` | `Choose(...)` |
| a choice, or a trigger's targets | `ChooseTarget { candidates, optional, count, any_number }` | `Target(...)` |
| modes, piles, options | `ChooseModes { modes: Vec<Label>, eligible, count }` | `Modes(...)` |
| colors | `ChooseColors { choices, count }` | `Colors(...)` |
| may | `May { purpose }` | `Accept` / `Decline` |
| unless a player pays | `g.unless_pays(...)` asks it | `PayCost` / `Decline` |
| distribute cards | `DistributeCards { cards, destinations, may_shuffle }` | `Arrange(... \| ...)` |
| search a library | `SearchLibrary` (use `mechanics::search_library`) | `Search(...)` |

Their defaults (what the runner answers when no line applies) are in `crates/cantrip-script/src/run.rs`, `default`.

**Words of its rules text: read through `words`.** A text-changing effect (CR 612: Trait Doctoring, Artificial
Evolution) replaces a color word, basic land type or creature type in an object's text, and the card then does what
the changed text says. So every color word, basic land type and creature type a card's rules text uses as one is read
through the words of the object whose text it is: `g.words(item.object).color(Color::Blue)` for "if it's blue",
`view.words(source).subtype(Subtype::Goblin)` in a static ability for "other Goblins", `q.words(asker.source)` in a
condition. Mana symbols (`{U}`) are not words, and a card's name is never changed (CR 612.2), nor is a keyword's
reminder text. The layers change the type line, and a token gets the colors and creature types of the text that makes
it without the card doing anything (CR 612.4); a token a keyword's own rules make (For Mirrodin!'s Rebel) is created
with `create_tokens_worded(.., Words::NONE)`, and so is every keyword's added later (afterlife's Spirits, fabricate's
Servos). An object that has left the battlefield keeps the words it last had
(`words` reads its last-known information), so a sacrificed source's ability reads them too.

## Asking about the past: the event history

Every rules event since the game began is in the history, in order: `q.history()` (`cantrip_api::history`). Each
`Record` has its serial, turn, step, whose turn it was, its cause, and the `Event`; records reference objects and copy
nothing out of them, so what an object was is read from its row (never deleted; frozen as it left) or its last-known
snapshot. A trigger matching an event, and a replacement deciding about one, see the history as it was just before
that event.

Any question about the past is answered there, never with a new field in the state:

1. **A one-off goes in the card's file.** Use `history.select(EventQuery::kind(..).player(..).this_turn()...)`, the
   typed helpers on `History` (`casts`, `casts_by`, `cast_of`, `drawn`, `life_gained`, `life_lost`, `resolutions`,
   `triggerings`, `done`, `discarded_this_turn`, `attacked_this_turn`), or a plain iterator over `history.this_turn()`
   or `history.all()` with whatever closure the card needs.
2. **A question a second printed card asks in the same words moves to `cantrip_mechanics::history`** as a named
   function, and both cards call it (`first_draw_in_own_draw_step` for Orcish Bowmasters and Hullbreacher,
   `permanent_left_battlefield_this_turn` for revolt, `put_into_graveyard_from_battlefield_this_turn`,
   `cards_left_graveyard_this_turn`, `spells_cast_this_turn` for storm, `opponents_who_lost_life_this_turn`,
   `entered_this_turn`, `drawn_this_turn`).
3. **The core only records.** If an event a card needs is not recorded, record it (an `Event` variant emitted where
   the rules do the thing, or `Cx::record` for one no ability triggers on); never add a counter.

**A card file that is only `CardDef { ..oracle::X }` is a stub**, and `crates/cantrip-cards/tests/no_stub_cards.rs` fails on
one for a card whose Oracle text is more than core keywords (a card that is all core keywords needs no file). The
exceptions are a draft-only card (`draftAbilities` in its JSON) and a file that starts a comment `// Characteristics only:`
saying why every line of the card's text is already read from its JSON.

## Where code goes

**A keyword is never written in a card's file.** Every keyword action (CR 701: earthbend, amass, adapt, manifest
dread, vote, ...) and keyword ability (CR 702: evoke, persist, ripple, fading, boast, backup, ...) is a function or
module in `cantrip-mechanics` named for it and parameterised, from the first card that has it, and the card calls it
(`mechanics::earthbend(g, target, 1)`). Look there first; if it is missing, add it there, citing the CR rule, and say
in its doc comment which printed cards use it. Only a keyword that changes a core procedure (trample, first strike,
split second) goes in the core.

The layering of [engine-design.md](engine-design.md) holds, and CLAUDE.md's rule with it:

1. **The card's file** for everything one card does. That is the default.
2. **`cantrip-mechanics`** for a recipe a second already-printed card would use exactly as written (connive, cycling,
   search, a target slot). Name the second card in the doc comment.
3. **`cantrip-rules`** (the core) only for rules authority: a procedure of the Comprehensive Rules, a kind of
   question, a cost type, a zone-change rule. Never a card's behavior. A core change is reviewed on its own.

Card crates depend on `cantrip-api`, `cantrip-mechanics` and `cantrip-oracle` only. The compiler enforces it.

## What already exists

Before writing anything, look at what the cards already ported do (`crates/cantrip-cards/src/cards/`). Most
shapes have an example:

- a targeted spell (Lightning Bolt);
- a choice from a revealed hand (Duress);
- "each player" choices in turn order (Liliana's +1);
- piles (Liliana's −6);
- an "unless" cost (Mana Leak, Miscalculation);
- alternative costs (Fireblast, Gush, Firebolt's flashback);
- cycling (Miscalculation);
- an ETB trigger (Satyr Wayfinder);
- a "becomes the target" trigger (Phantasmal Image's exception);
- a copy as it enters (Phantasmal Image);
- an any-color mana ability (Black Lotus);
- a stack ability with a mid-resolution color choice (Deathrite Shaman);
- library search and shuffle (Fabled Passage, Demonic Tutor);
- look and arrange (Ponder);
- a planeswalker (Liliana).

## Designations, dungeons and the rules that limit casting

- **The monarch and the initiative** (CR 725, 726) are `Player.monarch` / `Player.initiative`; a card says
  `g.become_monarch(p)` or `g.take_initiative(p)`. Their inherent triggers (the end-step draw, the steal by combat
  damage, the venture on taking the initiative and at upkeep) are the rules' own abilities (`cantrip_api::intrinsic`),
  put on the stack from an object in no zone (`designations.rs`). `Query` reads `state().player(p).monarch`.
- **Venturing** (CR 701.49, 309) walks `Player.room` through a dungeon card (`CardDef.dungeon`: the arrows of each room;
  the ability of the room is the ability of the same index, its label "Name — text"). The Undercity is
  `cards/undercity.rs`; `g.venture(p)` enters or moves on, and the rules complete it (CR 704.5t, `dungeons_completed`).
- **Revolt** is `cantrip_mechanics::history::permanent_left_battlefield_this_turn(q, p)`. **Harness** (CR 701.64) is an effect:
  `g.add_effect(id, id, EffectKind::Harnessed, Duration::Indefinite)`, read by a static that grants the infinity ability
  (The Mind Stone). `Duration::UntilNextTurnOf(p)` is "until your next turn".
- **Cost changes** are `CardDef.cost_changes` (Thalia-likes, Bilbo, Anointed Peacekeeper); **cast limits and bans** are
  `CardDef.cast_rules` (`fn(&dyn Query, &Asker, caster) -> bool`, false forbids: Teferi); **as though it had flash** is
  `EffectKind::CastAsThoughFlash(types)` for a resolved spell's effect and `CardDef.flash_rules` for a static ability
  ("You may cast Aura and Equipment spells as though they had flash", Sigarda's Aid); a static lifting a loyalty ability's timing is `CardDef.instant_activation`
  (The Wandering Emperor). The board is searched for these only when something is cast or activated.
- **Mana**: `PlayGrant.any_type` / `CastTerms.any_type` (mana of any type pays, CR 609.4b); mana that can't be spent on
  nonartifact spells is `g.add_restricted_mana(..)` from an `ActivatedDef { restricted: Some(..) }` ability (CR 106.6).
- **Gift** (CR 702.174) is `CardDef.gift`; the promise is asked with the spell, and the spell's effect reads
  `item.gift.is_some()`. **Player hexproof** is `EffectKind::PlayerHexproof`.
- **Names and players**: `RequestKind::ChooseName` (a card name, answered `Answer::Name`) and `ChoosePlayer`; a name chosen as
  a permanent enters is `Entering.chosen_card`, kept in `Obj.marks.chosen_card`.
- **Modal triggered abilities** are `TriggeredDef.modal`; **playing from the top of a library for life** is
  `PlayRule::LibraryTop` (Bolas's Citadel).

## Rules bricks added with the second wave of cards

- **Entering with counters** (CR 122.6, 614.1c): `Game::put_onto_battlefield_with_counters` and
  `create_tokens_with_counters`; `mechanics::tokens::incubate` (CR 701.53). **`Game::win_game`** (CR 104.2b).
- **Combat**: planeswalkers are attackable (`DeclareAttackers.planeswalkers`); `Keyword::CantBeAttacked` (CR 508.1c),
  `CantBeBlockedByPowerTwoOrLess`, `CantBeBlockedExceptByThreeOrMore` (`Chars::minimum_blockers`, menace's two too).
  A scenario refuses an attack or a block with `! Bot2.Attack(A > B)` / `! Bot2.Block(A > B)`.
- **Counters**: `Event::CountersPut` carries `by` (CR 122.6a), the player who put them. **Costs**: `Cost.unattach_self`.
- **Turns**: `DelayedWhen::UntilNextTurnOf(p)`; `CardDef.turn_begin` skips a turn (Time Vault, CR 614.10).
- **Spells**: copies of permanent spells become tokens (CR 707.10f), `Event::Copied` (magecraft); abilities a static grants
  to a spell on the stack are seen by the cast-trigger scan; `Game::play_land_during_resolution`; `mechanics::hideaway`.
- **Mana**: an `{X}` in a mana ability's cost is announced and read as `Obj::x` of its source; an `any_color` ability with
  no color to choose is still offered and adds nothing (CR 106.5).
- **Layers and rules**: `Layer::PtSwitch` / `EffectKind::SwitchPt` (CR 613.4d), `EffectKind::PlayerProtectionFromEverything`
  (CR 702.16j), `CardDef.unpreventable_damage` (CR 615.12), `CardDef.cause_rules` (what an opponent's spells and
  abilities can't make you discard or sacrifice, CR 101.2), `Query::event_subject` for a trigger's target filter, and
  miracle (`CardDef.miracle`, CR 702.94).
