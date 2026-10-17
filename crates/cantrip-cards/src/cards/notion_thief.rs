use cantrip_api::*;
use cantrip_mechanics::replacements::{instead_of_drawing, opponent_would_draw_except_first_in_draw_step};
use cantrip_oracle as oracle;

/// "... instead that player skips that draw and you draw a card": the opponent's draw is not made (CR 614.6), and Notion
/// Thief's controller draws one, which meets every draw replacement effect that has not yet had its opportunity at the
/// opponent's draw (CR 614.5) - their own, or another player's Notion Thief, so that with one on each side the card comes
/// back to the player who would have drawn it (its ruling). The opponent's other instructions still happen.
fn you_draw(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    g.draw(asker.controller, 1);
}

pub static CARD: CardDef = CardDef {
    // "If an opponent would draw a card except the first one they draw in each of their draw steps, instead that player
    // skips that draw and you draw a card." (CR 121.6, 614.1a-b)
    replacements: &[replacement(
        ReplacementKind::Draw,
        opponent_would_draw_except_first_in_draw_step,
        instead_of_drawing,
        "skips the draw, and this creature's controller draws a card instead",
    )
    .then(you_draw)],
    ..oracle::NOTION_THIEF
};
