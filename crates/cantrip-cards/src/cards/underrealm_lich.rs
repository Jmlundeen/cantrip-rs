use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_indestructible_then_tap;
use cantrip_mechanics::replacements::{instead_of_drawing, you_would_draw_a_card};
use cantrip_mechanics::library::look_at_top_take_some_rest_to_graveyard;
use cantrip_oracle as oracle;

/// "... instead look at the top three cards of your library, then put one into your hand and the rest into your
/// graveyard" (CR 614.1a, 401.2): no card is drawn (CR 121.5), so nothing that triggers on drawing does. A library of fewer
/// than three shows what it has (CR 609.3), and an empty one shows nothing: the draw is replaced all the same, so its
/// player does not lose for it (its ruling).
fn look_instead(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    let _ = look_at_top_take_some_rest_to_graveyard(g, asker.controller, asker.source, 3, 1);
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(life(4), this_gains_indestructible_then_tap, "Pay 4 life: This creature gains indestructible until end of turn. Tap it.")],
    // "If you would draw a card, instead look at the top three cards of your library, then put one into your hand and the
    // rest into your graveyard." (CR 121.6, 614.1a)
    replacements: &[replacement(
        ReplacementKind::Draw,
        you_would_draw_a_card,
        instead_of_drawing,
        "puts one of the top three cards into hand instead, and the rest into the graveyard",
    )
    .then(look_instead)],
    ..oracle::UNDERREALM_LICH
};
