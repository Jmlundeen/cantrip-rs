//! A trigger on its own card being discarded functions in every zone the discarded card can wind up in (CR 113.6k,
//! 702.29c), a library included: Wheel of Sun and Moon reveals a card bound for its owner's graveyard and puts it on the
//! bottom of their library, and "when you discard this card" (Titanbones, Towering Heart) and "when you cycle this card"
//! (the cycling decrees) trigger from there. Library of Leng puts a card discarded by an effect on top of its owner's
//! library unrevealed: its characteristics are undefined (CR 701.9c), its abilities among them (CR 109.3), and it triggers
//! on nothing. No printed card in the engine has such a trigger or puts a discarded card into a library yet, so test-only
//! ones do (`cantrip_script::run_on`).

use std::sync::OnceLock;

use cantrip_engine::*;
use cantrip_script::{GameScript, run_on};

/// "When you discard this card": the discard of this very card, wherever it went (CR 701.9a).
fn this_discarded(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { card, .. } if *card == source => Fired::PLAIN,
        _ => None,
    }
}

fn gain_three(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    Ok(())
}

/// A 2/2 with "When you discard this card, you gain 3 life."
static BONES: CardDef = CardDef {
    name: "Probe Bones",
    oracle: true,
    has_cost: true,
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    abilities: &[triggered(when(EventMask::DISCARDED, this_discarded), gain_three, "When you discard this card, you gain 3 life.")
        .functions_in(WHERE_A_DISCARD_ENDS)],
    ..CardDef::BLANK
};

/// "If a card would be discarded, put it on top of its owner's library instead of into their graveyard" (Library of
/// Leng's replacement, without its choice): the card is not revealed.
fn discarded_to_a_graveyard(_: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.reason == ZoneReason::Discard && z.to.is(ZoneKind::Graveyard))
}

fn library_instead(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::library(q.obj(z.object).owner);
    }
}

static LENG: CardDef = CardDef {
    name: "Probe Leng",
    oracle: true,
    has_cost: true,
    types: TypeSet::of(&[CardType::Artifact]),
    replacements: &[replacement(ReplacementKind::ZoneChange, discarded_to_a_graveyard, library_instead, "puts a discarded card on top of its owner's library")],
    ..CardDef::BLANK
};

/// "... reveal it and put it on top of its owner's library instead": the same, revealed as it goes (CR 701.20a).
fn library_instead_revealed(q: &dyn Query, asker: &Asker, proposal: &mut Proposal) {
    library_instead(q, asker, proposal);
    if let Proposal::ZoneChange(z) = proposal {
        z.reveal = true;
    }
}

static REVEALING_LENG: CardDef = CardDef {
    name: "Probe Revealing Leng",
    replacements: &[replacement(ReplacementKind::ZoneChange, discarded_to_a_graveyard, library_instead_revealed, "reveals a discarded card and puts it on top of its owner's library")],
    ..LENG
};

/// Probe Bones that, as Blightsteel Colossus, is revealed and shuffled into its owner's library instead of going to a
/// graveyard from anywhere.
static SHUFFLING_BONES: CardDef = CardDef {
    name: "Probe Shuffling Bones",
    replacements: &[cantrip_mechanics::replacements::REVEAL_AND_SHUFFLE_INTO_LIBRARY_INSTEAD_OF_GRAVEYARD],
    ..BONES
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&BONES, &LENG, &REVEALING_LENG, &SHUFFLING_BONES])))
}

fn play(name: &str, text: &str) {
    let script = GameScript::parse(text, name).unwrap_or_else(|e| panic!("{name}: {e}"));
    if let Err(failure) = run_on(&script, table()) {
        panic!("{name}:\n{}", failure.report);
    }
}

/// Faithless Looting discards Probe Bones and an Island, and the Leng puts both on top of the library unrevealed: the
/// Bones' characteristics are undefined there (CR 701.9c), and its "when you discard this card" does not trigger. (Its
/// revealed twin below triggers.)
#[test]
fn a_card_discarded_unrevealed_into_a_library_does_not_trigger() {
    play("discard-trigger-not-from-a-hidden-library", &script("Probe Leng", "? Stack.Count = 0", 20));
}

/// The same discard, with the card revealed as it is put on top of the library: it triggers from there (CR 113.6k), 3 life.
/// A second Probe Bones already in the library is not the card discarded and does not trigger, nor does the Island.
#[test]
fn a_card_discarded_revealed_into_a_library_triggers_from_there() {
    play("discard-trigger-from-a-library", &script("Probe Revealing Leng", "? Stack.Count = 1", 23));
}

/// Faithless Looting under `leng`, discarding Probe Bones and an Island into the library: the stack checked after the
/// discard, and Bot1's life at the end.
fn script(leng: &str, stack: &str, life: u32) -> String {
    format!(
        "
[Bot1 Deck]
{leng}
Mountain
Faithless Looting
Probe Bones
Island
Island
Island
Island
Island
Probe Bones
Island x12

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mountain.Play
Bot1.Battlefield.Mountain.Activate
Bot1.Hand.Faithless Looting.Cast
Bot1.Pass
Bot1.Choose(Probe Bones; Island)
{stack}
? Bot1.Library.Count = 15
Bot1.Pass
Bot2.Pass

[Expect]
Bot1.Life = {life}
Bot1.Library.Count = 15
Bot1.Graveyard = Faithless Looting
Bot1.Hand.Count = 4
"
    )
}

/// Faithless Looting discards Probe Shuffling Bones, which is revealed and shuffled into the library instead of reaching
/// the graveyard: its trigger functions in the library it was shuffled into, and 3 life.
#[test]
fn a_card_discarded_and_shuffled_into_a_library_triggers_from_there() {
    play(
        "discard-trigger-from-a-shuffled-library",
        "
[Bot1 Deck]
Mountain
Faithless Looting
Probe Shuffling Bones
Island
Island
Island
Island
Island
Island
Island x12

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.Mountain.Play
Bot1.Battlefield.Mountain.Activate
Bot1.Hand.Faithless Looting.Cast
Bot1.Pass
Bot1.Choose(Probe Shuffling Bones; Island)
? Stack.Count = 1
Bot1.Pass
Bot2.Pass

[Expect]
Bot1.Life = 23
Bot1.Library.Count = 13
Bot1.Graveyard = Faithless Looting; Island
",
    );
}

/// Aclazotz's "whenever an opponent discards a land card" reads the discarded card (CR 701.9a), which the Leng put on top of
/// its owner's library: unrevealed it is undefined and no land card (CR 701.9c), so no Bat; revealed, two lands are two Bats.
#[test]
fn an_opponents_land_discarded_unrevealed_into_a_library_is_no_land_card_to_aclazotz() {
    play("aclazotz-hidden-discard", &aclazotz("Probe Leng", 0));
}

#[test]
fn an_opponents_land_discarded_revealed_into_a_library_is_a_land_card_to_aclazotz() {
    play("aclazotz-revealed-discard", &aclazotz("Probe Revealing Leng", 2));
}

/// Bot1 puts the Leng and Aclazotz onto the battlefield; Bot2 loots away two Islands under it.
fn aclazotz(leng: &str, bats: u32) -> String {
    format!(
        "
[Bot1 Deck]
{leng}
Black Lotus
Mox Jet
Mox Ruby
Aclazotz, Deepest Betrayal
Island x15

[Bot2 Deck]
Mountain
Faithless Looting
Island
Island
Island
Island
Island
Island x13

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Hand.Mox Jet.Cast
Bot1.Hand.Mox Ruby.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Black)
Bot1.Battlefield.Mox Jet.Activate
Bot1.Battlefield.Mox Ruby.Activate
Bot1.Hand.Aclazotz, Deepest Betrayal.Cast
Bot1.Pass
Bot2.Pass
[T2 Bot2.Main1]
Bot2.Hand.Mountain.Play
Bot2.Battlefield.Mountain.Activate
Bot2.Hand.Faithless Looting.Cast
Bot2.Pass
Bot2.Choose(Island; Island)
Bot2.Pass
Bot1.Pass

[Expect]
Bot1.Battlefield.Count = {count}
Bot2.Library.Count = 12
Bot2.Graveyard = Faithless Looting
",
        count = 4 + bats
    )
}

/// Seasoned Pyromancer discards a Lightning Bolt and an Island, which the Leng puts on top of the library: unrevealed, the
/// Bolt is undefined and no nonland card (CR 701.9c), so no Elemental; revealed, it is one, and one Elemental.
#[test]
fn a_card_discarded_unrevealed_into_a_library_is_no_nonland_card_to_seasoned_pyromancer() {
    play("pyromancer-hidden-discard", &pyromancer("Probe Leng", 0));
}

#[test]
fn a_card_discarded_revealed_into_a_library_is_a_nonland_card_to_seasoned_pyromancer() {
    play("pyromancer-revealed-discard", &pyromancer("Probe Revealing Leng", 1));
}

fn pyromancer(leng: &str, elementals: u32) -> String {
    format!(
        "
[Bot1 Deck]
{leng}
Black Lotus
Seasoned Pyromancer
Lightning Bolt
Island
Island
Island
Island x15

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Red)
Bot1.Hand.Seasoned Pyromancer.Cast
Bot1.Pass
Bot2.Pass
Bot1.Choose(Lightning Bolt; Island)
Bot1.Pass
Bot2.Pass

[Expect]
Bot1.Battlefield.Count = {count}
Bot1.Library.Count = 15
",
        count = 2 + elementals
    )
}

/// A pair of scripts under the hidden and the revealing Leng: `make(leng, revealed)`.
fn both(name: &str, make: fn(&str, bool) -> String) {
    play(&format!("{name}-hidden"), &make("Probe Leng", false));
    play(&format!("{name}-revealed"), &make("Probe Revealing Leng", true));
}

/// Ledger Shredder connives on Bot1's second spell of turn 3 and discards a Lightning Bolt into the library: unrevealed it is
/// undefined and no nonland card (CR 701.9c), so no counter (CR 701.50a); revealed, one.
#[test]
fn connive_counts_no_nonland_card_discarded_unrevealed_into_a_library() {
    both("connive", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Black Lotus
Ledger Shredder
Ornithopter
Ornithopter
Lightning Bolt
Island
Island x15

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Blue)
Bot1.Hand.Ledger Shredder.Cast
Bot1.Pass
Bot2.Pass
[T3 Bot1.Main1]
Bot1.Hand.Ornithopter.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Ornithopter.Cast
Bot1.Choose(Lightning Bolt)
Bot1.Pass
Bot2.Pass
[T3 Bot1.Main2]

[Expect]
Bot1.Battlefield.Ledger Shredder.Counters.PlusOnePlusOne = {counters}
Bot1.Graveyard = Black Lotus
",
            counters = if revealed { "1" } else { "(absent)" }
        )
    });
}

/// Seasoned Cryomancer discards a Lightning Bolt and an Island into the library: unrevealed, no nonland card was discarded
/// this way (CR 701.9c) and the reflexive ability does not trigger; revealed, it taps and stuns the Ornithopter.
#[test]
fn seasoned_cryomancer_sees_no_nonland_card_discarded_unrevealed_into_a_library() {
    both("cryomancer", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Ornithopter
Black Lotus
Mox Sapphire
Seasoned Cryomancer
Island
Island
Lightning Bolt
Island
Island x15

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Ornithopter.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Sapphire.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Blue)
Bot1.Battlefield.Mox Sapphire.Activate
Bot1.Hand.Seasoned Cryomancer.Cast
Bot1.Pass
Bot2.Pass
Bot1.Choose(Lightning Bolt; Island)
{target}
[Bot1.Main2]

[Expect]
Bot1.Battlefield.Ornithopter.Tapped = {revealed}
Bot1.Graveyard = Black Lotus
",
            // Unrevealed, the reflexive ability's "up to that many" targets are never asked for.
            target = if revealed { "Bot1.Target(Ornithopter)" } else { "! Bot1.Target" }
        )
    });
}

/// Glacial Dragonhunt discards a Lightning Bolt into the library: unrevealed it is no nonland card (CR 701.9c) and the
/// Ornithopter lives; revealed, the reflexive ability deals it 3 damage.
#[test]
fn glacial_dragonhunt_sees_no_nonland_card_discarded_unrevealed_into_a_library() {
    both("dragonhunt", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Mox Sapphire
Mox Ruby
Ornithopter
Glacial Dragonhunt
Lightning Bolt
Island
Island x15

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Sapphire.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Ruby.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Ornithopter.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Mox Sapphire.Activate
Bot1.Battlefield.Mox Ruby.Activate
Bot1.Hand.Glacial Dragonhunt.Cast
Bot1.Pass
Bot2.Pass
Bot1.Choose(Lightning Bolt)
{target}
[Bot1.Main2]

[Expect]
Bot1.Battlefield.Count = {count}
",
            // Its only legal target, the Ornithopter, is chosen for it.
            target = if revealed { "" } else { "? Stack.Count = 0" },
            count = if revealed { 3 } else { 4 }
        )
    });
}

/// Garruk's −3 has Bot2 discard a Lightning Bolt and a Counterspell into the library: unrevealed they are no nonland cards
/// (CR 701.9c), so Bot2 "didn't discard two nonland cards this way" and Bot1 draws; revealed, Bot1 draws nothing.
#[test]
fn garruk_counts_no_nonland_card_discarded_unrevealed_into_a_library() {
    both("garruk", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Black Lotus
Mox Jet
Mox Jet
Garruk, Veiled Butcher
Forest
Forest
Forest x15

[Bot2 Deck]
Lightning Bolt
Counterspell
Island x18

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Jet.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Jet.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Black)
Bot1.Battlefield.Mox Jet.Activate
Bot1.Battlefield.Mox Jet.Activate
Bot1.Hand.Garruk, Veiled Butcher.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Garruk, Veiled Butcher.Activate(Ability: Each opponent discards)
Bot1.Pass
Bot2.Pass
Bot2.Choose(Lightning Bolt; Counterspell)
[Bot1.Main2]

[Expect]
Bot1.Hand.Count = {hand}
Bot2.Library.Count = 15
",
            hand = if revealed { 2 } else { 3 }
        )
    });
}

/// Occult Epiphany (X = 3) discards an Ornithopter and two Islands into the library: unrevealed they are undefined and add
/// no card type (CR 701.9c), so no Spirit; revealed, artifact, creature and land make three.
#[test]
fn occult_epiphany_counts_no_type_of_a_card_discarded_unrevealed_into_a_library() {
    both("epiphany", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Black Lotus
Mox Sapphire
Occult Epiphany
Island
Island
Island
Island
Ornithopter
Island
Island x10

[Bot2 Deck]
Swamp x15

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Pass
Bot1.Hand.Mox Sapphire.Cast
Bot1.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Blue)
Bot1.Battlefield.Mox Sapphire.Activate
Bot1.Hand.Occult Epiphany.Cast(X: 3)
Bot1.Choose(Ornithopter; Island; Island)
Bot1.Pass
[Bot1.Main2]

[Expect]
Bot1.Battlefield.Count = {count}
",
            count = if revealed { 5 } else { 2 }
        )
    });
}

/// Ill-Timed Explosion discards a Lightning Bolt and a Counterspell into the library: unrevealed they have no mana value (CR
/// 701.9c), so X is 0 and the Ornithopter (0/2) lives; revealed, X is 2 and it dies.
#[test]
fn ill_timed_explosion_reads_no_mana_value_of_a_card_discarded_unrevealed_into_a_library() {
    both("explosion", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Mox Sapphire
Mox Ruby
Mox Jet
Mox Pearl
Ill-Timed Explosion
Ornithopter
Lightning Bolt
Counterspell
Plains x10

[Bot2 Deck]
Plains x15

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Ornithopter.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Sapphire.Cast
Bot1.Hand.Mox Ruby.Cast
Bot1.Hand.Mox Jet.Cast
Bot1.Hand.Mox Pearl.Cast
Bot1.Battlefield.Mox Sapphire.Activate
Bot1.Battlefield.Mox Ruby.Activate
Bot1.Battlefield.Mox Jet.Activate
Bot1.Battlefield.Mox Pearl.Activate
Bot1.Hand.Ill-Timed Explosion.Cast
Bot1.Accept
Bot1.Choose(Lightning Bolt; Counterspell)
? Stack.Count = 1
Bot1.Pass
Bot2.Pass
[Bot1.Main2]

[Expect]
Bot1.Battlefield.Count = {count}
Bot1.Library.Count = 12
",
            count = if revealed { 5 } else { 6 }
        )
    });
}

/// Recruit (The Queen of Dale, on Bot2's Lightning Bolt) discards an Ornithopter into the library: unrevealed it is no
/// nonland card (CR 701.9c), so no Human Soldier (CR 701.70a); revealed, one.
#[test]
fn recruit_sees_no_nonland_card_discarded_unrevealed_into_a_library() {
    both("recruit", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Plains
Mox Pearl
The Queen of Dale
Ornithopter
Island
Island
Island x20

[Bot2 Deck]
Mountain
Lightning Bolt
Mountain x5
Mountain x20

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Plains.Play
Bot1.Hand.Mox Pearl.Cast
Bot1.Pass
Bot1.Battlefield.Plains.Activate
Bot1.Battlefield.Mox Pearl.Activate
Bot1.Hand.The Queen of Dale.Cast
Bot1.Pass
[T2 Bot2.Main1]
Bot2.Hand.Mountain.Play
Bot2.Battlefield.Mountain.Activate
Bot2.Hand.Lightning Bolt.Cast(Bot1)
Bot2.Pass
Bot1.Choose(Ornithopter)
Bot2.Pass
Bot1.Pass
[T2 Bot2.Main2]

[Expect]
Bot1.Life = 17
Bot1.Battlefield.Count = {count}
Bot1.Graveyard.Count = 0
",
            count = if revealed { 5 } else { 4 }
        )
    });
}

/// Hollow Marauder has Bot2 discard a Force of Will (mana value 5) into the library: unrevealed it is undefined and has no
/// mana value (CR 701.9c), so Bot2 "didn't discard a card with mana value 4 or greater" and Bot1 draws; revealed, no draw.
#[test]
fn hollow_marauder_reads_no_mana_value_of_a_card_discarded_unrevealed_into_a_library() {
    both("marauder", |leng, revealed| {
        format!(
            "
[Bot1 Deck]
{leng}
Black Lotus
Mox Jet
Mox Ruby
Mox Emerald
Mox Pearl
Hollow Marauder
Swamp x13

[Bot2 Deck]
Force of Will
Island x19

[Actions]
Bot1.Hand.{leng}.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Black Lotus.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Jet.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Ruby.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Emerald.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Pearl.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Black Lotus.Activate(Color: Black)
Bot1.Battlefield.Mox Jet.Activate
Bot1.Battlefield.Mox Ruby.Activate
Bot1.Battlefield.Mox Emerald.Activate
Bot1.Battlefield.Mox Pearl.Activate
Bot1.Hand.Hollow Marauder.Cast
Bot1.Pass
Bot2.Pass
Bot1.Target(Bot2)
Bot1.Pass
Bot2.Pass
Bot2.Choose(Force of Will)
[Bot1.Main2]

[Expect]
Bot1.Hand.Count = {hand}
Bot2.Library.Count = 14
",
            hand = if revealed { 0 } else { 1 }
        )
    });
}
