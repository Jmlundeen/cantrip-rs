use cantrip_api::*;
use cantrip_mechanics::history::faced_a_dilemma;
use cantrip_mechanics::{choose, library, targets, triggers::EACH_END_STEP};
use cantrip_oracle as oracle;

/// "If you faced a dilemma this turn" (CR 700.2a, 700.2b): an intervening "if" (CR 603.4), asked of the event history
/// for the ability's own controller - choosing no mode is no dilemma, and a copy chooses none.
fn faced_one_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().this_turn().iter().any(|r| faced_a_dilemma(&r.event) == Some(asker.controller))
}

/// "Target opponent exiles cards from the top of their library until they exile a nonland card. You may cast that card
/// without paying its mana cost" (CR 701.13a, 607.2a, 608.2g, 118.9): the nonland card this resolution's exiling stopped on is the
/// only one `cast_up_to_free` offers - not the lands around it, nor a card an earlier resolution exiled.
fn exile_until_nonland_then_cast_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let by = g.linked(item);
    let Some(card) = library::exile_until_any_nonland(g, opponent, Some(by)) else { return Ok(()) };
    choose::cast_up_to_free(g, item.controller, item.object, &[card], 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        EACH_END_STEP,
        exile_until_nonland_then_cast_it,
        "At the beginning of each end step, if you faced a dilemma this turn, target opponent exiles cards from the top of their library until they exile a nonland card. You may cast that card without paying its mana cost.",
    )
    .targets(&[targets::OPPONENT])
    .condition(faced_one_this_turn)],
    ..oracle::THE_CARETAKER
};
