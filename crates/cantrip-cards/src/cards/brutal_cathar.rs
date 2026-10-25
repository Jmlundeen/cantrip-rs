use cantrip_api::*;
use cantrip_mechanics::triggers::{ward_life, THIS_ENTERS_OR_TRANSFORMS};
use cantrip_mechanics::{exile_target_until_leaves_effect, targets};
use cantrip_oracle as oracle;

pub static BACK: CardDef = CardDef {
    abilities: &[ward_life::<3>("Ward\u{2014}Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.)")],
    ..oracle::MOONRAGE_BRUTE
};

/// "Whenever this creature enters or transforms into Brutal Cathar, exile target creature an opponent controls until this
/// creature leaves the battlefield." (CR 610.3; nothing is exiled if it already left, 610.3b.)
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS_OR_TRANSFORMS,
        exile_target_until_leaves_effect,
        "Whenever this creature enters or transforms into Brutal Cathar, exile target creature an opponent controls until this creature leaves the battlefield.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    back: Some(&BACK),
    ..oracle::BRUTAL_CATHAR
};
