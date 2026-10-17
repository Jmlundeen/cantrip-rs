//! Dredge (CR 702.52): "Dredge N" means "As long as you have at least N cards in your library, if you would draw a card, you
//! may instead mill N cards and return this card from your graveyard to your hand." A static ability that functions only
//! while the card is in a graveyard (CR 702.52a), its owner's "you". Golgari Grave-Troll (dredge 6) and Stinkweed Imp
//! (dredge 5) print it.

use cantrip_api::*;

use crate::replacements::instead_of_drawing;

/// "As long as you have at least N cards in your library, if you would draw a card" (CR 702.52a-b): one draw of the card's
/// owner, judged as it is about to be made (CR 121.2), with N or more cards to mill. One draw is replaced by one dredge
/// ability at most, the draw being gone once one has (CR 614.6).
fn applies<const N: u8>(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Draw(d) if d.player == asker.controller && q.state().zone(Zone::library(d.player)).len() >= N as usize)
}

/// "... mill N cards and return this card from your graveyard to your hand": the mill first, then the card; a card a
/// replacement effect kept from the graveyard is no longer this card there, and is not returned (CR 400.7).
fn mill_and_return<const N: u8>(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    g.mill(asker.controller, N as u32);
    g.return_to_hand(&[asker.source]);
}

/// What the owner is asked as their draw would be replaced (CR 614.1a: "you may instead"), N cards to mill its amount.
const fn question(ask: &'static str, n: u8) -> Question {
    Question::new("Dredge", ask, "If you don't, the draw goes on as it was.", "Dredge", "Don't dredge").amount(n).no_default()
}

static QUESTIONS: [Question; 6] = [
    question("dredge 1 - mill a card and return {source} to your hand instead of drawing a card?", 1),
    question("dredge 2 - mill two cards and return {source} to your hand instead of drawing a card?", 2),
    question("dredge 3 - mill three cards and return {source} to your hand instead of drawing a card?", 3),
    question("dredge 4 - mill four cards and return {source} to your hand instead of drawing a card?", 4),
    question("dredge 5 - mill five cards and return {source} to your hand instead of drawing a card?", 5),
    question("dredge 6 - mill six cards and return {source} to your hand instead of drawing a card?", 6),
];

static HINTS: [&str; 6] = [
    "may mill a card and return this card to hand instead (dredge 1)",
    "may mill two cards and return this card to hand instead (dredge 2)",
    "may mill three cards and return this card to hand instead (dredge 3)",
    "may mill four cards and return this card to hand instead (dredge 4)",
    "may mill five cards and return this card to hand instead (dredge 5)",
    "may mill six cards and return this card to hand instead (dredge 6)",
];

/// "Dredge N" (CR 702.52a), N from 1 to 6, as a card's replacement effect.
pub const fn dredge<const N: u8>() -> ReplacementDef {
    assert!(N >= 1 && N <= 6, "dredge 1 to dredge 6 are printed");
    replacement(ReplacementKind::Draw, applies::<N>, instead_of_drawing, HINTS[N as usize - 1])
        .then(mill_and_return::<N>)
        .optional(&QUESTIONS[N as usize - 1])
        .in_graveyard()
}
