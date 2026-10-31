use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, your_turn};
use cantrip_oracle as oracle;

/// Mine Collapse deals 5 damage to target creature or planeswalker.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ways: &[way(Cost::sacrifice(1, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>), ZoneKind::Hand, "Sacrifice a Mountain")
        .condition(your_turn)],
    ..oracle::MINE_COLLAPSE
};
