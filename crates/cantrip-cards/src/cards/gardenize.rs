use cantrip_api::*;
use cantrip_mechanics::triggers::{CREATURE_YOU_CONTROL_DIES, YOUR_FIRST_MAIN_PHASE};
use cantrip_oracle as oracle;

/// "Whenever a creature you control dies, put a charge counter on this enchantment" (CR 603.6c).
fn add_charge(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
}

/// "At the beginning of your first main phase, add {G} for each charge counter on this enchantment" (CR 106.4).
fn add_green(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let n = g.obj(item.object).counters(CounterKind::Charge);
    g.add_mana(item.controller, Mana::G, n.max(0) as u8);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            CREATURE_YOU_CONTROL_DIES,
            add_charge,
            "Whenever a creature you control dies, put a charge counter on this enchantment.",
        ),
        triggered(
            YOUR_FIRST_MAIN_PHASE,
            add_green,
            "At the beginning of your first main phase, add {G} for each charge counter on this enchantment.",
        ),
    ],
    ..oracle::GARDENIZE
};
