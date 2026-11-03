use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, target opponent discards a card." (CR 701.9): their choice; nothing if they are no longer a legal
/// target as the ability resolves (CR 608.2b).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, effects::target_player_discards::<1>, "When this creature enters, target opponent discards a card.")
        .targets(&[targets::OPPONENT])],
    ..oracle::CORRUPT_COURT_OFFICIAL
};
