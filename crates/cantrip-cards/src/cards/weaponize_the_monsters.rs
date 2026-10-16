use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_oracle as oracle;

/// This enchantment deals 2 damage to any target.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(sacrifice(1, cantrip_mechanics::targets::filters::creature)), effect, "{2}, Sacrifice a creature: This enchantment deals 2 damage to any target.")
        .targets(&[cantrip_mechanics::targets::ANY])],
    ..oracle::WEAPONIZE_THE_MONSTERS
};
