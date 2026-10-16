use cantrip_api::*;
use cantrip_mechanics::{damage_target, harmonize, targets};
use cantrip_oracle as oracle;

/// "Channeled Dragonfire deals 2 damage to any target."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ways: &[harmonize::way(mana("{5}{R}{R}"), "Harmonize {5}{R}{R}")],
    ..oracle::CHANNELED_DRAGONFIRE
};
