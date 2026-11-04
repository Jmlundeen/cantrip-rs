use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, statics, station, targets};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::damage_first_target::<3>, "When this Spacecraft enters, it deals 3 damage to any target.")
            .targets(&[targets::ANY]),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)",
        ),
        ability(cost("{1}{R}"), effects::this_gets::<2, 0>, "{1}{R}: This Spacecraft gets +2/+0 until end of turn."),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<1, 5>, EIGHT_PLUS).continues(),
    ],
    ..oracle::DEBRIS_FIELD_CRUSHER
};
