use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 1), "When this land enters, you gain 1 life."),
        mana_ability(TAP, add_one_of(&[Color::Red, Color::White]), "{T}: Add {R} or {W}."),
    ],
    ..oracle::WIND_SCARRED_CRAG
};
