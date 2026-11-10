use cantrip_api::*;
use cantrip_mechanics::{station, statics, tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "When this Spacecraft enters, create two 1/1 colorless Borg artifact creature tokens" (CR 603.6a, 111.1, 111.2): both
        // tokens are created at once, as one event, under the ability's controller. The shared body.
        triggered(THIS_ENTERS, tokens::create_borg::<2>, "When this Spacecraft enters, create two 1/1 colorless Borg artifact creature tokens."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        // "{7+}[abilities][P/T]" (CR 721.2a-b): while it has seven or more charge counters it is an artifact creature with base
        // power and toughness 4/4 in addition to its other types, and has the striation's static ability.
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<4, 4>, SEVEN_PLUS).continues(),
        // "Artifact creatures you control get +1/+1" (CR 604.1, 611.2c): its own ability, on whatever the card is, so it pumps
        // the Cube itself once the 7+ striation has made it an artifact creature (CR 613.1f, 613.4c).
        static_ability(
            Layer::PtModify,
            statics::your_artifact_creatures,
            statics::plus_pt::<1, 1>,
            "Artifact creatures you control get +1/+1.",
        ),
    ],
    ..oracle::CLASS_4_TACTICAL_CUBE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
