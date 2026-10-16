use cantrip_api::*;
use cantrip_mechanics::{effects, solved, triggers};
use cantrip_oracle as oracle;

/// "To solve — You have no cards in hand" (CR 719.3a): an intervening "if" (CR 603.4), true as the end step begins and as the
/// ability resolves.
fn no_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::hand(asker.controller)).is_empty()
}

/// "Solved — At the beginning of your upkeep" (CR 702.169c): "This ability triggers only if this Case is solved."
fn your_upkeep_while_solved(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if solved::is_solved(q, me.source) { triggers::your_upkeep(q, me, event) } else { None }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, |g, item| effects::discard_then_draw(g, item, 2), "When this Case enters, discard a card, then draw two cards."),
        triggered(solved::TO_SOLVE, solved::become_solved, "To solve \u{2014} You have no cards in hand. (If unsolved, solve at the beginning of your end step.)")
            .condition(no_cards_in_hand),
        triggered(when_asked(EventMask::STEP_BEGAN, your_upkeep_while_solved), |g, item| effects::discard_hand_then_draw(g, item, 2), "Solved \u{2014} At the beginning of your upkeep, discard your hand, then draw two cards."),
    ],
    delayed: &[solved::MARKER],
    ..oracle::CASE_OF_THE_CRIMSON_PULSE
};
