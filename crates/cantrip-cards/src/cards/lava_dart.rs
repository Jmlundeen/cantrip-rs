use cantrip_api::*;
use cantrip_mechanics::{damage_target, flashback, targets};
use cantrip_oracle as oracle;

/// "Lava Dart deals 1 damage to any target."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    // "Flashback—Sacrifice a Mountain." (CR 702.34a)
    ways: &[flashback::way(Cost::sacrifice(1, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>), "Flashback—Sacrifice a Mountain")],
    ..oracle::LAVA_DART
};
