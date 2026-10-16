use cantrip_api::*;
use cantrip_mechanics::{effects, solved, triggers};
use cantrip_oracle as oracle;

/// "To solve — You have no cards in hand" (CR 719.3a): an intervening "if" (CR 603.4), true as the end step begins and as the
/// ability resolves.
fn no_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::hand(asker.controller)).is_empty()
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, |g, item| effects::discard_then_draw(g, item, 2), "When this Case enters, discard a card, then draw two cards."),
        solved::to_solve(no_cards_in_hand, "To solve \u{2014} You have no cards in hand. (If unsolved, solve at the beginning of your end step.)"),
        solved::triggered(triggers::YOUR_UPKEEP, |g, item| effects::discard_hand_then_draw(g, item, 2), "Solved \u{2014} At the beginning of your upkeep, discard your hand, then draw two cards."),
    ],
    ..oracle::CASE_OF_THE_CRIMSON_PULSE
};
