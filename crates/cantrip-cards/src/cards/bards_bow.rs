use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::job_select::JOB_SELECT;
use cantrip_mechanics::statics::{have_reach, plus_pt};
use cantrip_mechanics::tokens::HERO_1_1_COLORLESS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[JOB_SELECT, equip(mana("{6}"), "Perseus's Bow \u{2014} Equip {6}")],
    statics: &[
        static_ability(Layer::Type, attached_object, cantrip_mechanics::statics::also_a::<{ cantrip_mechanics::lands::sub(Subtype::Bard) }>, "Equipped creature is a Bard in addition to its other types."),
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
        static_ability(Layer::Ability, attached_object, have_reach, "Equipped creature has reach."),
    ],
    ..oracle::BARDS_BOW
};

pub static EXTRA: &[&CardDef] = &[&HERO_1_1_COLORLESS];
