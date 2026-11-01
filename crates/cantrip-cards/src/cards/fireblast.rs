use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{effects, targets};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<4>),
    ways: &[way(Cost::sacrifice(2, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>), ZoneKind::Hand, "Sacrifice two Mountains")],
    ..oracle::FIREBLAST
};
