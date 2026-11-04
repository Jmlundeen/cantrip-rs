use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// The 5+ striation, "Flying" with the 3/2 power/toughness box (CR 721.2a-b): while it has five or more charge counters it
/// has flying and is a creature with base power and toughness 3/2 in addition to its other types. One static ability,
/// applied in each layer it touches (CR 613.6).
const FIVE_PLUS: &str = "5+ | Flying (3/2)";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this Spacecraft enters, draw a card."),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 5+.)",
        ),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<5>, station::is_a_creature, FIVE_PLUS),
        static_ability(Layer::Ability, station::charged::<5>, statics::have_flying, FIVE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<5>, statics::base_pt::<3, 2>, FIVE_PLUS).continues(),
    ],
    ..oracle::SUUROK_CLASS_SCIENCE_VESSEL
};
