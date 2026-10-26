use cantrip_api::*;
use cantrip_mechanics::{effects, mana, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(THIS_ENTERS, effects::gain_one_life, "When this land enters, you gain 1 life."),
        mana_ability(TAP, add_one_of(&[Color::Blue, Color::Red]), "{T}: Add {U} or {R}."),
    ],
    ..oracle::ANDORIA
};
