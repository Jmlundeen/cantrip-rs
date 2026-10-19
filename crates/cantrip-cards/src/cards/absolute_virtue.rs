use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "This spell can't be countered." and "Flying" are core keywords (CR 101.2, 702.9).
    // "You have protection from each of your opponents." (CR 702.16i, 702.16k)
    player_protection: &[statics::YOU_HAVE_PROTECTION_FROM_EACH_OPPONENT],
    ..oracle::ABSOLUTE_VIRTUE
};
