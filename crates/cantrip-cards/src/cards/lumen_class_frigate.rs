use cantrip_api::*;
use cantrip_mechanics::{statics, station};
use cantrip_oracle as oracle;

const TWO_PLUS: &str = "2+ | Other creatures you control get +1/+1.";
const TWELVE_PLUS: &str = "12+ | Flying, lifelink";

/// "2+ | Other creatures you control get +1/+1." (CR 721.2a, 613.4c): the striation applies while this Spacecraft has two or
/// more charge counters, to every other creature its controller controls.
fn other_creatures_while_two_charged(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    station::charge(q, source) >= 2 && statics::other_creatures_you_control(q, source, affected, so_far)
}

/// "12+ | Flying, lifelink": both keywords in layer 6 (CR 613.1f, 702.9, 702.15), one part of the effect.
fn have_flying_and_lifelink(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, source, chars);
    statics::have_lifelink(q, source, chars);
}

/// Lifelink is granted by the 12+ striation, not printed on the card, so it is not a core keyword here (CR 721.2a).
pub static CARD: CardDef = CardDef {
    abilities: &[station::ability(
        "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 12+.)",
    )],
    statics: &[
        static_ability(Layer::PtModify, other_creatures_while_two_charged, statics::plus_pt::<1, 1>, TWO_PLUS),
        static_ability(Layer::Type, station::charged::<12>, station::is_a_creature, TWELVE_PLUS),
        static_ability(Layer::Ability, station::charged::<12>, have_flying_and_lifelink, TWELVE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<12>, statics::base_pt::<3, 5>, TWELVE_PLUS).continues(),
    ],
    ..oracle::LUMEN_CLASS_FRIGATE
};
