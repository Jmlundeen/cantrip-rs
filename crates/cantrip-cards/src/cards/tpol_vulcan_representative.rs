use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When T'Pol enters, tap target creature an opponent controls and put two stun counters on it." (CR 122.1d, 701.26a):
/// the shared tap_and_stun generic Rowdy Snowballers calls with <1>, here with <2>.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::tap_and_stun::<2>,
        "When T'Pol enters, tap target creature an opponent controls and put two stun counters on it. (If a permanent with a stun counter would become untapped, remove one from it instead.)",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::TPOL_VULCAN_REPRESENTATIVE
};
