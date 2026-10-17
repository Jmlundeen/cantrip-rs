use cantrip_api::*;
use cantrip_oracle as oracle;

/// This enchantment deals 1 damage to each opponent.
fn strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::damage_each_opponent(g, item, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[cantrip_mechanics::statics::CREATURES_YOU_CONTROL_GET_PLUS_ONE],
    abilities: &[triggered(cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_ENTERS, strike, "Whenever a creature you control enters, this enchantment deals 1 damage to each opponent.")],
    ..oracle::WARLEADERS_CALL
};
