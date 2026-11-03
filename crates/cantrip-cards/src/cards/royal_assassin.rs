use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// The target must be tapped when chosen and when the ability resolves (CR 115.1c, 608.2b).
pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, effects::destroy_first_target, "{T}: Destroy target tapped creature.")
        .targets(&[targets::TAPPED_CREATURE])],
    ..oracle::ROYAL_ASSASSIN
};
