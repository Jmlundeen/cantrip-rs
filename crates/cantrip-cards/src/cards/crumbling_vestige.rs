use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(THIS_ENTERS, effects::add_one_mana_of_any_color, "When this land enters, add one mana of any color."),
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
    ],
    ..oracle::CRUMBLING_VESTIGE
};
