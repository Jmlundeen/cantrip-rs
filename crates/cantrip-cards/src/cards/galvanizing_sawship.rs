use cantrip_api::*;
use cantrip_mechanics::{statics, station};
use cantrip_oracle as oracle;

const THREE_PLUS: &str = "3+ | Flying, haste";

/// "Flying, haste" are granted by the 3+ striation, not printed (CR 721.2a): a creature only while it has three charge
/// counters, and in one layer 6 part that continues the layer 4 type change (CR 613.6).
pub static CARD: CardDef = CardDef {
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 3+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<3>, station::is_a_creature, THREE_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying_and_haste, THREE_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<6, 5>, THREE_PLUS).continues(),
    ],
    ..oracle::GALVANIZING_SAWSHIP
};
