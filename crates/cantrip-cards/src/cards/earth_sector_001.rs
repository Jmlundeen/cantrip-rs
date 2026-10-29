use cantrip_api::*;
use cantrip_mechanics::{effects, mana, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(THIS_ENTERS, effects::gain_one_life, "When this land enters, you gain 1 life."),
        mana_ability(TAP, add_one_of(&[Color::Green, Color::Blue]), "{T}: Add {G} or {U}."),
    ],
    ..oracle::EARTH_SECTOR_001
};
