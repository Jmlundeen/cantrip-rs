use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "If this card is in your opening hand, you may begin the game with it on the battlefield" (CR 103.6) and
/// "You have hexproof" (CR 702.11c), a player keyword ability of the permanent.
pub static CARD: CardDef = CardDef {
    opening_hand: Some(statics::BEGIN_ON_THE_BATTLEFIELD),
    player_keywords: &[statics::YOU_HAVE_HEXPROOF],
    ..oracle::LEYLINE_OF_SANCTITY
};
