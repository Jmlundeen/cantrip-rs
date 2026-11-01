use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, mana, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(THIS_ENTERS, effects::damage_first_target::<1>, "When this land enters, it deals 1 damage to target opponent.").targets(&[targets::OPPONENT]),
        mana_ability(TAP, add_one_of(&[Color::Black, Color::Green]), "{T}: Add {B} or {G}."),
    ],
    ..oracle::FESTERING_GULCH
};
