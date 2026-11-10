use cantrip_api::*;
use cantrip_mechanics::{station, statics};
use cantrip_oracle as oracle;

/// "During your turn, this Spacecraft and creatures you control have hexproof" (CR 611.3a, 613.1f, 702.11b): the source
/// itself and each creature its controller controls, while it is that player's turn. "This Spacecraft" is the source
/// whether or not it is a creature yet, so the test on the source comes first and is not a creature test.
fn your_spacecraft_and_creatures_on_your_turn(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    chars: &Chars,
) -> bool {
    let you = q.state().obj(source).controller;
    if q.state().turn.active != you || q.state().obj(affected).controller != you {
        return false;
    }
    affected == source || chars.is(CardType::Creature)
}

/// The 4+ striation, "Flying, menace, haste" with the 5/3 power/toughness box (CR 721.2b): while it has four or more
/// charge counters it is a creature with base power and toughness 5/3 in addition to its other types - an artifact
/// creature, as its reminder text says - and it has flying, menace and haste. One static ability, applied in each layer
/// it touches (CR 613.6).
const FOUR_PLUS: &str = "4+ | Flying, menace, haste (5/3)";

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            your_spacecraft_and_creatures_on_your_turn,
            statics::have_hexproof,
            "During your turn, this Spacecraft and creatures you control have hexproof.",
        ),
        static_ability(Layer::Type, station::charged::<4>, station::is_a_creature, FOUR_PLUS),
        static_ability(Layer::Ability, station::charged::<4>, statics::have_flying_menace_and_haste, FOUR_PLUS)
            .continues(),
        static_ability(Layer::PtSet, station::charged::<4>, statics::base_pt::<5, 3>, FOUR_PLUS).continues(),
    ],
    abilities: &[station::ability(
        "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 4+.)",
    )],
    ..oracle::KVORT_CLASS_RAIDER
};
