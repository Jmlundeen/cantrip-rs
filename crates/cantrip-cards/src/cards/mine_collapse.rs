use cantrip_api::*;
use cantrip_mechanics::{effects, targets, your_turn};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effects::damage_first_target::<5>),
    ways: &[way(Cost::sacrifice(1, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>), ZoneKind::Hand, "Sacrifice a Mountain")
        .condition(your_turn)],
    ..oracle::MINE_COLLAPSE
};
