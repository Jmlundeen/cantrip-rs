use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::job_select::JOB_SELECT;
use cantrip_mechanics::tokens::HERO_1_1_COLORLESS;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[JOB_SELECT, equip(mana("{2}"), "Equip {2}")],
    statics: &[
        static_ability(Layer::Type, attached_object, cantrip_mechanics::statics::also_a::<{ cantrip_mechanics::lands::sub(Subtype::Monk) }>, "Equipped creature is a Monk in addition to its other types."),
        static_ability(Layer::PtModify, attached_object, plus_pt::<1, 0>, "Equipped creature gets +1/+0."),
    ],
    ..oracle::MONKS_FIST
};

pub static EXTRA: &[&CardDef] = &[&HERO_1_1_COLORLESS];
