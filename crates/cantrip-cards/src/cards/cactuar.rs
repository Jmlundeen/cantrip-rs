use cantrip_api::*;
use cantrip_mechanics::history::entered_this_turn;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// The intervening "if" (CR 603.4): this creature, the object now on the battlefield, did not enter it this turn.
fn did_not_enter_this_turn(q: &dyn Query, me: &Asker) -> bool {
    !entered_this_turn(q.history(), me.source)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        effects::return_this_to_hand,
        "At the beginning of your end step, if this creature didn't enter the battlefield this turn, return it to its owner's hand.",
    )
    .condition(did_not_enter_this_turn)],
    ..oracle::CACTUAR
};
