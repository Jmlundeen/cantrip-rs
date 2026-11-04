use cantrip_api::*;
use cantrip_mechanics::{each_opponent_sacrifices, statics, station, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

/// "Each opponent sacrifices a nontoken creature of their choice" (CR 701.21a, 101.4).
fn opponents_sacrifice_a_nontoken_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| q.chars(id).is(CardType::Creature) && !q.obj(id).token)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            opponents_sacrifice_a_nontoken_creature,
            "When this Spacecraft enters, each opponent sacrifices a nontoken creature of their choice.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<4, 3>, SEVEN_PLUS).continues(),
    ],
    ..oracle::SUSURIAN_DIRGECRAFT
};
