use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, targets, triggers};
use cantrip_oracle as oracle;

/// The 8+ striation, "Flying" with the 4/4 power/toughness box (CR 721.2b): while it has eight or more charge counters it
/// has flying and is a creature with base power and toughness 4/4 in addition to its other types.
const EIGHT_PLUS: &str = "8+ | Flying (4/4)";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            effects::destroy_first_target,
            "When U.S.S. Excelsior enters, destroy target tapped creature an opponent controls.",
        )
        .targets(&[targets::TAPPED_CREATURE_AN_OPPONENT_CONTROLS]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, station::charged::<8>, statics::have_flying, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<8>, statics::base_pt::<4, 4>, EIGHT_PLUS).continues(),
    ],
    ..oracle::U_S_S_EXCELSIOR_CUTTING_EDGE
};
