//! A scenario's `Divide(...)` answers the division an activated ability announces as it is activated (CR 601.2d by way of
//! 602.2b), as it answers a spell's and a triggered ability's: a loyalty ability's counters, and an ability's damage among a
//! player and a creature. `! Divide(...)` refuses a division the question does not take. No printed card in the engine has
//! such an ability yet (Vivien, Arkbow Ranger, Arc Mage and Samut, the Tested will), so test-only ones do
//! (`cantrip_script::run_on`), beside the real cards.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_script::{GameScript, run_on};

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Creature))
}

fn distribute(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::counters_divided(g, item, CounterKind::PlusOne);
    Ok(())
}

fn damage(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::damage_divided(g, item);
    Ok(())
}

/// "Probe Ranger", a {0} planeswalker with loyalty 3: "+1: Distribute two +1/+1 counters among up to two target creatures."
/// and "−2: Probe Ranger deals 2 damage divided as you choose among one or two targets."
static RANGER: CardDef = CardDef {
    name: "Probe Ranger",
    oracle: true,
    has_cost: true,
    types: TypeSet::of(&[CardType::Planeswalker]),
    loyalty: Some(3),
    abilities: &[
        ability(loyalty(1), distribute, "+1: Distribute two +1/+1 counters among up to two target creatures.")
            .targets(&[up_to(2, creature)])
            .divide(Divide { total: 2, reads: None }),
        ability(loyalty(-2), damage, "−2: Probe Ranger deals 2 damage divided as you choose among one or two targets.")
            .targets(&[cantrip_mechanics::targets::ANY.between(1, 2)])
            .divide(Divide { total: 2, reads: None }),
    ],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&RANGER])))
}

fn play(name: &str, text: &str) {
    let script = GameScript::parse(text, name).unwrap_or_else(|e| panic!("{name}: {e}"));
    if let Err(failure) = run_on(&script, table()) {
        panic!("{name}:\n{}", failure.report);
    }
}

/// The +1 puts one counter on each Ornithopter of Bot1's; the next turn the −2 deals 1 to Bot2 and 1 to Bot2's Ornithopter,
/// refusing first a division that gives the player all of it and the Ornithopter nothing.
#[test]
fn a_loyalty_ability_is_divided_by_the_scenario() {
    play(
        "a loyalty ability divided",
        "
[Bot1 Deck]
Probe Ranger
Ornithopter
Ornithopter
Island x27

[Bot2 Deck]
Ornithopter
Island x29

[Actions]
Bot1.Hand.Ornithopter#1.Cast
Bot1.Hand.Ornithopter.Cast
Bot1.Hand.Probe Ranger.Cast
Bot1.Battlefield.Probe Ranger.Activate(Ability: +1)
Bot1.Divide(Bot1.Battlefield.Ornithopter#1: 1; Bot1.Battlefield.Ornithopter#2: 1)
[T2 Bot2.Main1]
Bot2.Hand.Ornithopter.Cast
[T3 Bot1.Main1]
Bot1.Battlefield.Probe Ranger.Activate(Ability: −2)
! Bot1.Divide(Bot2: 2; Bot2.Battlefield.Ornithopter: 0)
Bot1.Divide(Bot2: 1; Bot2.Battlefield.Ornithopter: 1)
[T3 Bot1.Main1]

[Expect]
Bot1.Battlefield.Ornithopter#1.Counters.PlusOnePlusOne = 1
Bot1.Battlefield.Ornithopter#2.Counters.PlusOnePlusOne = 1
Bot1.Battlefield.Probe Ranger.Counters.Loyalty = 2
Bot2.Life = 19
Bot2.Battlefield.Ornithopter.Damage = 1
",
    );
}
