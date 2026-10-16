use cantrip_api::*;
use cantrip_mechanics::{attach::equip, triggers};
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on each creature you control. If you have the city's blessing, put two +1/+1 counters on each creature
/// you control instead: read as the ability resolves.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let n = if g.state().player(you).citys_blessing { 2 } else { 1 };
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, you, CounterKind::PlusOne, n);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ascend: true,
    abilities: &[
        triggered(triggers::EQUIPPED_CREATURE_ATTACKS, counters, "Whenever equipped creature attacks, put a +1/+1 counter on each creature you control. If you have the city's blessing, put two +1/+1 counters on each creature you control instead."),
        equip(mana("{3}"), "Equip {3}"),
    ],
    ..oracle::AND_RIL_NARSIL_REFORGED
};
