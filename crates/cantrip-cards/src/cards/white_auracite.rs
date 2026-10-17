use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves_effect, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        triggered(THIS_ENTERS, exile_target_until_leaves_effect, "When this artifact enters, exile target nonland permanent an opponent controls until this artifact leaves the battlefield.")
            .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::WHITE_AURACITE
};
