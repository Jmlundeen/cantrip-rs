use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, triggers};
use cantrip_oracle as oracle;

/// The 4+ striation, "Flying" with the 2/2 power/toughness box (CR 721.2b): while it has four or more charge counters it
/// has flying and is a creature with base power and toughness 2/2 in addition to its other types - an artifact creature,
/// as its reminder text says. One static ability, applied in each layer it touches (CR 613.6).
const FOUR_PLUS: &str = "4+ | Flying (2/2)";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, |g, item| effects::surveil(g, item, 2), "When this Spacecraft enters, surveil 2."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 4+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<4>, station::is_a_creature, FOUR_PLUS),
        static_ability(Layer::Ability, station::charged::<4>, statics::have_flying, FOUR_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<4>, statics::base_pt::<2, 2>, FOUR_PLUS).continues(),
    ],
    ..oracle::WURMWALL_SWEEPER
};
