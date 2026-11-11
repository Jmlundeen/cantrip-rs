use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_DIES;
use cantrip_mechanics::{effects, station, statics};
use cantrip_oracle as oracle;

const THREE_PLUS: &str = "3+ | Flying";

const ENTERS_OR_DIES: &str = "When U.S.S. Enterprise-C enters or dies, you may discard your hand. If you do, draw a card for each card discarded this way.";

const ASK: Question = Question::new(
    "Discard",
    "discard your hand?",
    "If you do, draw a card for each card discarded this way",
    "Discard, then draw",
    "Keep your hand",
);

/// "You may discard your hand. If you do, draw a card for each card discarded this way" (CR 608.2c-d, 701.9, 121.1): the
/// controller is asked as the ability resolves and the whole hand goes at once there and then; how many cards that was is
/// counted before any card is drawn, so a hand of three draws three and an empty hand draws none.
fn may_discard_hand_to_draw_that_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(discarded) = effects::may_discard_hand(g, item, &ASK)? {
        g.draw(item.controller, discarded);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS_OR_DIES, may_discard_hand_to_draw_that_many, ENTERS_OR_DIES),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 3+.)",
        ),
    ],
    statics: &[
        // "3+|[abilities][P/T]" (CR 721.2a-b): while it has three or more charge counters it is an artifact creature with base
        // power and toughness 6/6 in addition to its other types, and has the striation's flying.
        static_ability(Layer::Type, station::charged::<3>, station::is_a_creature, THREE_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, THREE_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<6, 6>, THREE_PLUS).continues(),
    ],
    ..oracle::U_S_S_ENTERPRISE_C_TIME_TOSSED
};
