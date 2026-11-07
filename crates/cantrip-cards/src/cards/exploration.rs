use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may play an additional land on each of your turns." (CR 305.2: cumulative with other such effects, the ruling.)
pub static CARD: CardDef = CardDef {
    play_rules: &[PlayRule::ExtraLandPlays(1)],
    ..oracle::EXPLORATION
};
