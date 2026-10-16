use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{triggers::THIS_ENTERS, warp};
use cantrip_oracle as oracle;

/// "As long as you have one or fewer cards in hand, if you would draw one or more cards" (CR 121.2a, 614.1a): asked as
/// the instruction is about to be carried out, never stored, and before any of its individual draws, so the cards it
/// adds cannot turn the condition off partway through. An instruction to draw none is not one to draw one or more.
fn would_draw(q: &dyn Query, asker: &Asker, p: &Proposal) -> bool {
    matches!(p, Proposal::DrawInstruction(d)
        if d.player == asker.controller && d.count >= 1 && q.state().zone(Zone::hand(asker.controller)).len() <= 1)
}

/// ... you draw that many cards plus one instead.
fn plus_one(_: &dyn Query, _: &Asker, p: &mut Proposal) {
    if let Proposal::DrawInstruction(d) = p {
        d.count += 1;
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::draw::<1>, "When this creature enters, draw a card.")],
    replacements: &[ReplacementDef {
        hint: "draws one more card",
        kind: ReplacementKind::DrawInstruction,
        applies: would_draw,
        apply: plus_one,
        ..ReplacementDef::BLANK
    }],
    ways: &[warp::way(mana(1, 0, 1, 0, 0, 0), "Warp {1}{U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::QUANTUM_RIDDLER
};
