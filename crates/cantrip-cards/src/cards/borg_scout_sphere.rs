use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const SIX_PLUS: &str = "6+ | Artifact creatures you control have flying.";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this Spacecraft enters, draw a card."),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)",
        ),
    ],
    statics: &[
        // "{6+}[abilities][P/T]" (CR 721.2a-b): while it has six or more charge counters it is an artifact creature with base
        // power and toughness 5/5 in addition to its other types, and has the striation's static ability.
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, station::charged::<6>, station::has::<0>, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<5, 5>, SIX_PLUS).continues(),
    ],
    // The 6+ striation's static ability (CR 721.2a), which this permanent has while it has six or more charge counters.
    delayed: &[granted_static_ability(Layer::Ability, statics::your_artifact_creatures, statics::have_flying, "Artifact creatures you control have flying.")],
    ..oracle::BORG_SCOUT_SPHERE
};
