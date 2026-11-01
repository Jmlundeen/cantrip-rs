use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[cantrip_mechanics::targets::NONCREATURE_SPELL], effects::counter_first_target_unless_pays::<2>),
    ..oracle::SPELL_PIERCE
};
