use cantrip_api::*;
use cantrip_mechanics::{effects, flashback, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<1>),
    // "Flashback—Sacrifice a Mountain." (CR 702.34a)
    ways: &[flashback::way(Cost::sacrifice(1, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>), "Flashback—Sacrifice a Mountain")],
    ..oracle::LAVA_DART
};
