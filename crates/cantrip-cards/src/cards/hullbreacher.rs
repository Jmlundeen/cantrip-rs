use cantrip_api::*;
use cantrip_mechanics::replacements::{instead_of_drawing, opponent_would_draw_except_first_in_draw_step};
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Instead you create a Treasure token": one for the draw that did not happen (CR 614.6). Each draw of an instruction to
/// draw several is replaced on its own (CR 121.6).
fn treasure(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    let _ = tokens::create_treasure(g, asker.controller);
}

pub static CARD: CardDef = CardDef {
    // "If an opponent would draw a card except the first one they draw in each of their draw steps, instead you create a
    // Treasure token."
    replacements: &[replacement(ReplacementKind::Draw, opponent_would_draw_except_first_in_draw_step, instead_of_drawing, "creates a Treasure token instead of the draw")
        .then(treasure)],
    ..oracle::HULLBREACHER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
