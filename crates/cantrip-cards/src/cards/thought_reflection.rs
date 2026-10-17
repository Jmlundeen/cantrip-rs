use cantrip_api::*;
use cantrip_mechanics::replacements::{instead_of_drawing, you_would_draw_a_card};
use cantrip_oracle as oracle;

/// "... draw two cards instead": two draws, made one at a time (CR 121.2), each meeting the replacement effects that have
/// not yet had their opportunity at the draw it replaced (CR 614.5): a second Thought Reflection doubles each, so one draw
/// becomes four (its ruling), and this one doubles neither.
fn draw_two(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    g.draw(asker.controller, 2);
}

pub static CARD: CardDef = CardDef {
    // "If you would draw a card, draw two cards instead." (CR 121.6, 614.1a)
    replacements: &[replacement(ReplacementKind::Draw, you_would_draw_a_card, instead_of_drawing, "draws two cards instead").then(draw_two)],
    ..oracle::THOUGHT_REFLECTION
};
