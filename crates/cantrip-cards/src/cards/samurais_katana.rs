use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::job_select::{HERO, JOB_SELECT};
use cantrip_mechanics::statics::{have_haste, have_trample, plus_pt};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[JOB_SELECT, equip(mana("{5}"), "Murasame \u{2014} Equip {5}")],
    statics: &[
        static_ability(Layer::Type, attached_object, cantrip_mechanics::statics::also_a::<{ cantrip_mechanics::lands::sub(Subtype::Samurai) }>, "Equipped creature is a Samurai in addition to its other types."),
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
        static_ability(Layer::Ability, attached_object, have_trample, "Equipped creature has trample."),
        static_ability(Layer::Ability, attached_object, have_haste, "Equipped creature has haste."),
    ],
    ..oracle::SAMURAIS_KATANA
};

pub static EXTRA: &[&CardDef] = &[&HERO];
