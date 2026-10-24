use cantrip_api::*;
use cantrip_mechanics::effects::{damage_each_creature_and_planeswalker, destroy_artifacts_with_mana_value_at_most};
use cantrip_oracle as oracle;

/// "Brotherhood's End deals 3 damage to each creature and each planeswalker."
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_each_creature_and_planeswalker(g, item, 3);
    Ok(())
}

/// "Destroy all artifacts with mana value 3 or less."
fn destroy_cheap_artifacts(g: &mut dyn Game, _item: &StackItem) -> Result<(), Suspend> {
    destroy_artifacts_with_mana_value_at_most(g, 3);
    Ok(())
}

/// Choose one (CR 700.2).
static MODAL: ModalDef = modes(&[
    mode(damage, "Brotherhood's End deals 3 damage to each creature and each planeswalker."),
    mode(destroy_cheap_artifacts, "Destroy all artifacts with mana value 3 or less."),
])
.choose(1, 1);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::BROTHERHOODS_END };
