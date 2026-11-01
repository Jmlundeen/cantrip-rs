use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target_unless_pays::<1>),
    ways: &[way(Cost::return_to_hand(1, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Island) }>), ZoneKind::Hand, "Return an Island you control to its owner's hand")],
    ..oracle::DAZE
};
