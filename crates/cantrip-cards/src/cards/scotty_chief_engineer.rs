use cantrip_api::*;
use cantrip_mechanics::{effects, targets, untapped_artifact_you_control};
use cantrip_oracle as oracle;

/// "Tap an untapped artifact you control: Target creature can't be blocked this turn." (CR 602.2a, 509.1b, 611.2a).
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        Cost::tap_untapped(1, untapped_artifact_you_control),
        effects::target_creature_cant_be_blocked_this_turn,
        "Tap an untapped artifact you control: Target creature can't be blocked this turn.",
    )
    .targets(&[targets::CREATURE])],
    ..oracle::SCOTTY_CHIEF_ENGINEER
};
