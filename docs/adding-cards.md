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
   in the order the rules and the cards already implemented do.
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
    spell: spell(&[targets::ANY], effect),
    ..oracle::LIGHTNING_BOLT
};
```

Abilities are written cost first, then what the ability does, then the printed text (`crates/cantrip-api/src/ability.rs`):

```rust
abilities: &[
    mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
    ability(cost("{2}{G}"), pump_team, "{2}{G}: Creatures you control get +0/+1 until end of turn."),
    ability(loyalty(1), plus_one, "+1: Target player draws two cards, then discards two cards.").targets(&[targets::PLAYER]),
    triggered(THIS_ENTERS, draw_a_card, "When this creature enters, draw a card."),
],
```

- **Costs**: `TAP`, `SACRIFICE_THIS`, `cost("{1}{U}")` (mana symbols as printed, read at compile time; `cost("{1}, {T}")`
  includes the tap), `loyalty(n)`, `life(n)`, `sacrifice(n, filter)`, `discard(n, filter)`, `remove_counters(kind, n)`,
  `DISCARD_THIS`, `EXILE_THIS`, `EXERT`, joined with `.and(..)`; `return_tapped_to_hand(n, filter)` is "return N tapped
  [permanents]", whose permanents are chosen once the mana is paid, so one tapped for mana while paying may be among
  them (web-slinging, `cantrip_mechanics::web_slinging::way(cost, text)`); anything rarer is a `Cost { .., ..Cost::NONE }`. A bare
  mana cost (equip, flashback, kicker, madness) is `mana("{2}{R}")`, or `generic(n)` for a number the card works out.
- **Mana**: `add(Mana::G)`, `add_n(Mana::C, 2)`, `add_each(&[Mana::W, Mana::U])`, `add_any_color(1)`,
  `add_one_of(&[Color::Red, Color::Green])`, `.only_for(restriction)`. What these list is what is added; a card whose
  mana ability does more writes the effect itself with `.by(f)` (the listing then only says what it can add) or
  `add_by(f)` when nothing can be listed (Gaea's Cradle).
- **Triggers**: `cantrip_mechanics::triggers` names the common ones (`THIS_ENTERS`, `THIS_DIES`, `YOUR_UPKEEP`, ...); a
  card's own is `when(EventMask::DAMAGE, my_match_fn)`. A Saga's chapter is `chapter(n, effect, text)`.
- **The rest** is a method after the call: `.targets(..)`, `.sorcery_speed()`, `.once_each_turn()`, `.condition(..)`
  (an activation restriction, or a trigger's intervening "if"), `.modal(..)`, `.look_back()`, `.anywhere()`, and the
  others in `ability.rs`. A loyalty ability is already sorcery speed (CR 606.3).
- **Shared effects with an argument** are called in place: `triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), ..)`.

The rest of a card is written the same way, one constructor each, the rare parts as methods:

```rust
spell: spell(&[targets::CREATURE], effect),                       // `spell(&[], effect)` for none
target(filter), up_to_one(filter), up_to(3, filter),               // target slots; `targets::X.optional()`, `.between(min, max)`, `.counted()`,
                                                                  // `.if_kicked(0, TargetSpec::ANY_NUMBER)` ("if this spell was kicked, instead choose any number of ...")
static MODAL: ModalDef = modes(&[mode(deal_3, "...").targets(&[targets::CREATURE]), mode(gain_3, "...")]).choose(1, 2);
statics: &[static_ability(Layer::PtModify, attached_object, plus_one, "Equipped creature gets +1/+1.")],
statics: &[characteristic_defining(Layer::PtDefine, power_is_creatures, "...")],
replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "shuffles it into its owner's library").own_everywhere()],
ways: &[way(cost("{1}{B}"), ZoneKind::Hand, "Pay {1}{B}; an opponent draws a card")],  // keywords: flashback::way(cost(..), text)
kicker: kicker(mana("{2}{B}")),                                   // `multikicker(..)`
```

Definitions a card brings with it (a token, a copy's exception) go in `pub static EXTRA: &[&CardDef]` in the same
file. The generator's constant name is the card name upper-cased with every run of non-alphanumerics as `_`.

What a `CardDef` can carry (see `crates/cantrip-api/src/card.rs`):

| Field | For |
| --- | --- |
| `spell: spell(targets, effect)` | an instant's or sorcery's effect, and its target slots |
| `abilities: &[AbilityDef]` | its activated, mana and triggered abilities, written as above. **Mana abilities first**: the C# engine numbers them first. |
| `ways: &[way(..)]` | alternative costs (flashback from the graveyard, Fireblast's sacrifice, Gush's return, web-slinging's tapped creature), way 1 onward; `instant_speed` offers one whenever an instant could be cast, whatever the card's type (sneak); `kind(WayKind::..)` names the keyword a card asks about ("cast using web-slinging": `web_slinging::was_cast_using_web_slinging`) |
| `enters: Some(fn)` | "as this enters" and "enters with": edits the `Entering` record (tapped, counters, a copy) before it is on the battlefield |
| `delayed: &[AbilityDef]` | abilities its effects set up to trigger later (`Game::delayed_trigger` with `AbilityRef::delayed(card, n)`), or give an object with `EffectKind::AddAbility`; a quoted static ability it gives is `granted_static(layer, apply, text)` (layer 7 only; its source is the object that has it) |
| `replacements: &[replacement(..)]` | its replacement effects (CR 614): a `hint`, the event kind (`ZoneChange`, `Counters`, `DrawInstruction`), `applies(q, asker, &Proposal)` and `apply(q, asker, &mut Proposal)`, both pure; `own_everywhere` for "if this card would ... from anywhere". Several that apply to one event are ordered by the affected player (`Replace(...)` in a scenario). The shared "+1/+1 counters, plus one" is in `cantrip_mechanics::replacements` |
| `cast_limit: N` | "each player can't cast more than N spells each turn" (CR 101.2), while it is on the battlefield |
| `trigger_rules: &[..]` | "[permanents] entering don't cause abilities to trigger" (CR 603.2), while it is on the battlefield: `fn(q, asker, &Event, &Triggering) -> bool`, true stops that ability triggering on that event. `cantrip_mechanics::statics::creatures_entering_dont_trigger` is Torpor Orb's; `statics::entering(event)` is the permanent an event put onto the battlefield |
| `additional_triggers: &[..]` | "that ability triggers an additional time" (CR 603.2d), while it is on the battlefield: the same `fn(q, asker, &Event, &Triggering) -> bool`, true makes that ability trigger once more on that event. `Triggering::of_a_permanent_controlled_by(asker.controller)` is "a triggered ability of a permanent you control"; `statics::permanent_entering_triggers_again` (Elesh Norn, Yarok), `creature_dying_triggers_again` (Teysa Karlov, Drivnod) and `creature_attacking_triggers_again` (Isshin) are shared |

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
- Ask what the rules ask, when they ask it, the way the cards already implemented do. A question with one possible answer
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
| search a library | `SearchLibrary` (use `mechanics::search_library`, which names where the cards found go: the question says it) | `Search(...)` |

What a player is told about a question is part of the card (non-negotiable 5 of the `add-cards` skill): a `TargetSpec`
with two slots says what each is for with `.labelled("Return target spell or permanent to its owner's hand")` (`{player}` stands
for the opponent of an each-opponent slot), a cast during resolution asks with the `Purpose` that says what it costs (`CastFree`,
`CastCopyFree`, `Cast` for one paid for), and a question is never asked with a `Purpose` that means something else.

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

Before writing anything, look at what the cards already implemented do (`crates/cantrip-cards/src/cards/`). Most
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
- **Cost changes** are `CardDef.cost_changes` (Thalia-likes, Bilbo, Anointed Peacekeeper), a spell's own
  `own_cost_changes` (affinity), and an ability's own `.cost_change(f)` ("this ability costs ... less", power-up). Each
  returns a `CostDelta`: `more(n)` generic, `less(n)` generic, or `less_mana(cost)` for an amount that may be colored,
  whose colored part comes off that color and any excess off generic (CR 118.7). "Activate only once" is `.once()`;
  **cast limits and bans** are
  `CardDef.cast_rules` (`fn(&dyn Query, &Asker, caster, from: ZoneKind) -> bool`, false forbids casting from that zone:
  Teferi, Drannith Magistrate's "from anywhere other than their hands"), and a resolved spell's bar on a player casting from
  some zones is `EffectKind::CantCastSpellsFrom(player, ZoneKinds)` (Avatar's Wrath), both read by the core wherever a
  cast is begun, during a resolution too (CR 101.2, 601.3); **as though it had flash** is
  `EffectKind::CastAsThoughFlash(types)` for a resolved spell's effect and `CardDef.flash_rules` for a static ability
  ("You may cast Aura and Equipment spells as though they had flash", Sigarda's Aid); a static lifting a loyalty ability's timing is `CardDef.instant_activation`
  (The Wandering Emperor), and a resolved spell's "you may activate loyalty abilities of [Jace] planeswalkers you control
  ... any time you could cast an instant" is `EffectKind::InstantLoyalty(Some(Subtype::Jace))` (Jace's Machinations; `None`
  for every planeswalker), which changes only the timing (CR 606.3). The board is searched for these only when something is cast or activated.
- **Mana**: `PlayGrant.any_type` / `CastTerms.any_type` (mana of any type pays, CR 609.4b); mana that can't be spent on
  nonartifact spells is `g.add_restricted_mana(..)` from a mana ability with `.only_for(..)` (CR 106.6).
- **Gift** (CR 702.174) is `CardDef.gift`, one of `cantrip_mechanics::gift`'s constants (a card, a Food, a tapped Fish, a
  Treasure, an Octopus, an extra turn); the promise and the opponent are asked before the targets, the spell's effect reads
  `item.gift.is_some()` and a target filter `gift::promised(q, asker.source)`. An instant or sorcery gives it as it
  resolves (the core); a permanent lists `gift::trigger(text)`. Tokens a gift makes are listed in the card's `EXTRA`. **Player hexproof** is `EffectKind::PlayerHexproof`.
- **Names and players**: `RequestKind::ChooseName` (a card name, answered `Answer::Name`; its `kind`, `NameKind::Any`,
  `Nonland` or `Land`, is what the name must be the name of, CR 201.4a, and the core refuses any other) and `ChoosePlayer`;
  a name chosen as a permanent enters is `Entering.chosen_card`, kept in `Obj.marks.chosen_card`, and one chosen for a
  permanent as its ability resolves ("when this land enters, choose a land card name", a Saga's "choose a card name") is
  `Game::remember_chosen_name`, kept there too (CR 607.2d): Petrified Hamlet, Medomai's Prophecy.
- **Modal abilities** are `.modal(..)`. A modal spell whose number of modes is decided by its optional additional cost
  or its kicker ("If this spell's additional cost was paid, choose both instead", Pyrrhic Strike) says so with
  `ModalDef::when_paid(min, max)`: the core then announces the cost before the modes (CR 601.2b) and offers paying an
  optional additional cost only when that many modes could be chosen (CR 700.2a); a kicker is not gated yet. **Playing from the top of a library for life** is
  `PlayRule::LibraryTop` (Bolas's Citadel).

## Rules bricks added with the second wave of cards

- **Entering with counters** (CR 122.6, 614.1c): `Game::put_onto_battlefield_with_counters` and
  `create_tokens_with_counters`; `mechanics::tokens::incubate` (CR 701.53). **`Game::win_game`** (CR 104.2b).
- **Combat**: planeswalkers are attackable (`DeclareAttackers.planeswalkers`); `Keyword::CantBeAttacked` (CR 508.1c),
  `CantBeBlockedByPowerTwoOrLess`, `CantBeBlockedExceptByThreeOrMore` (`Chars::minimum_blockers`, menace's two too).
  A scenario refuses an attack or a block with `! Bot2.Attack(A > B)` / `! Bot2.Block(A > B)`.
- **Reflexive triggers** (CR 603.12, "when you do"): `Game::reflexive_trigger`; one whose quantity is what the first part
  did ("tap up to that many target creatures") is `Game::reflexive_trigger_counted(.., count)`, and its slot's
  `TargetSpec::COUNTED` bound and its stack item's `x` are that count (Seasoned Cryomancer, Miasma Demon, Loamcrafter
  Faun, Swashbuckler Extraordinaire).
- **Counters**: `Event::CountersPut` carries `by` (CR 122.6a), the player who put them. **Costs**: `Cost.unattach_self`.
- **Turns**: `DelayedWhen::UntilNextTurnOf(p)`; `CardDef.turn_begin` skips a turn (Time Vault, CR 614.10); "end the
  turn" is `g.end_the_turn()?` (CR 724.1, Time Stop), after which the rest of the effect still runs (Glorious End), and
  "the player whose turn it is may end the turn" asks with `Purpose::EndTurn` (Obeka). "Activate only during your turn"
  is `cantrip_mechanics::your_turn`.
- **Spells**: copies of permanent spells become tokens (CR 707.10f), `Event::Copied` (magecraft); abilities a static grants
  to a spell on the stack are seen by the cast-trigger scan; `Game::play_land_during_resolution`; `mechanics::hideaway`.
- **Mana**: an `{X}` in a mana ability's cost is announced and read as `Obj::x` of its source; an `any_color` ability with
  no color to choose is still offered and adds nothing (CR 106.5).
- **Layers and rules**: `Layer::PtSwitch` / `EffectKind::SwitchPt` (CR 613.4d), `EffectKind::PlayerProtectionFromEverything`
  (CR 702.16j), `CardDef.unpreventable_damage` (CR 615.12), `CardDef.cause_rules` (what an opponent's spells and
  abilities can't make you discard or sacrifice, CR 101.2), `Query::event_subject` for a trigger's target filter, and
  miracle (`CardDef.miracle`, CR 702.94).
