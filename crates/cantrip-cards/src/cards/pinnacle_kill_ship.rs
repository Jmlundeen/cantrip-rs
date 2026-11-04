use cantrip_api::*;
use cantrip_mechanics::{effects, statics, station, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::damage_first_target::<10>, "When this Spacecraft enters, it deals 10 damage to up to one target creature.")
            .targets(&[targets::CREATURE.between(0, 1)]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<7, 7>, SEVEN_PLUS).continues(),
    ],
    ..oracle::PINNACLE_KILL_SHIP
};
