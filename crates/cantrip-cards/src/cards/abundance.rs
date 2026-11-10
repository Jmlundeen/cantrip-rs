use cantrip_api::*;
use cantrip_mechanics::library::{self, BottomOrder};
use cantrip_mechanics::replacements::{instead_of_drawing, you_would_draw_a_card};
use cantrip_oracle as oracle;

/// "You may instead choose land or nonland and reveal cards from the top of your library until you reveal a card of the
/// chosen kind. Put that card into your hand and put all other cards revealed this way on the bottom of your library in any
/// order." (CR 121.6, 614.1a, 701.20, 401.4) The draw is replaced, never also taken; the card put into the hand is not drawn
/// (CR 121.5).
fn reveal_instead(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    let card = g.card_id(&CARD);
    let _ = library::reveal_until_land_or_nonland(g, asker.controller, asker.source, card, BottomOrder::Chosen);
}

pub static CARD: CardDef = CardDef {
    // "If you would draw a card, you may instead ..." (CR 121.6, 614.1a)
    replacements: &[replacement(
        ReplacementKind::Draw,
        you_would_draw_a_card,
        instead_of_drawing,
        "may reveal until a land or nonland card and put it into hand instead",
    )
    .then(reveal_instead)
    .optional(&library::REVEAL_UNTIL_KIND)],
    labels: &["Land", "Nonland"],
    ..oracle::ABUNDANCE
};
