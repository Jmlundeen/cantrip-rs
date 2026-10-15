use cantrip_api::*;
use cantrip_mechanics::{attach::equip, triggers};
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on each creature you control. If you have the city's blessing, put two +1/+1 counters on each creature
/// you control instead: read as the ability resolves.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let n = if g.state().player(you).citys_blessing { 2 } else { 1 };
    for id in g.permanents_of(you) {
        if g.chars(id).is(CardType::Creature) {
            g.add_counters(id, CounterKind::PlusOne, n);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ascend: true,
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever equipped creature attacks, put a +1/+1 counter on each creature you control. If you have the city's blessing, put two +1/+1 counters on each creature you control instead.",
            listens: EventMask::ATTACKS,
            matches: triggers::equipped_creature_attacks,
            effect: counters,
            ..TriggeredDef::BLANK
        }),
        equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}"),
    ],
    ..oracle::AND_RIL_NARSIL_REFORGED
};
