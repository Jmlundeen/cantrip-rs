use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_oracle as oracle;

/// This enchantment deals 2 damage to any target.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{2}, Sacrifice a creature: This enchantment deals 2 damage to any target.",
        cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), ..Cost::sacrifice(1, cantrip_mechanics::targets::filters::creature) },
        targets: &[cantrip_mechanics::targets::ANY],
        effect,
        ..ActivatedDef::BLANK
    })],
    ..oracle::WEAPONIZE_THE_MONSTERS
};
