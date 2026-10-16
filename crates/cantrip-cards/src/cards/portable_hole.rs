use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves_effect, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, exile_target_until_leaves_effect, "When this artifact enters, exile target nonland permanent an opponent controls with mana value 2 or less until this artifact leaves the battlefield.")
        .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS_WITH_MANA_VALUE_2_OR_LESS])],
    ..oracle::PORTABLE_HOLE
};
