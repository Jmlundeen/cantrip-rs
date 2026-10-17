use cantrip_api::*;
use cantrip_mechanics::history::first_draw_in_own_draw_step;
use cantrip_mechanics::replacements::{instead_of_drawing, you_would_draw_a_card, GAIN_TWICE_THAT_MUCH_LIFE};
use cantrip_oracle as oracle;

/// "If you would draw a card except the first one you draw in each of your draw steps" (CR 121.6, 614.1a): each draw on its
/// own, judged as it is about to be made, so the turn's first card drawn in the draw step is left alone, and the one after
/// it is not.
fn you_would_draw_except_first_in_draw_step(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    you_would_draw_a_card(q, asker, proposal) && !first_draw_in_own_draw_step(q, asker.controller)
}

/// "... draw two cards instead": two draws, each meeting the replacement effects that have not yet had their opportunity
/// at the draw this replaced (CR 614.5), so two Archives make four (its ruling).
fn draw_two(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    g.draw(asker.controller, 2);
}

pub static CARD: CardDef = CardDef {
    replacements: &[
        // "If you would gain life, you gain twice that much life instead."
        GAIN_TWICE_THAT_MUCH_LIFE,
        // "If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead."
        replacement(ReplacementKind::Draw, you_would_draw_except_first_in_draw_step, instead_of_drawing, "draws two cards instead")
            .then(draw_two),
    ],
    ..oracle::ALHAMMARRETS_ARCHIVE
};
