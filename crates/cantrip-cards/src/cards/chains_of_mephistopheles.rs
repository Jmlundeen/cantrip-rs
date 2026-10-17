use cantrip_api::*;
use cantrip_mechanics::choose_from_hand;
use cantrip_mechanics::history::first_draw_in_own_draw_step;
use cantrip_mechanics::replacements::instead_of_drawing;
use cantrip_oracle as oracle;

/// "If a player would draw a card except the first one they draw in each of their draw steps" (CR 121.6, 614.1a): any
/// player's draw, each on its own (CR 121.2), judged as it is about to be made; the first card a player draws in their own
/// draw step is left alone (a draw replaced before it was not drawn, CR 614.6).
fn a_player_would_draw(q: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Draw(d) if !first_draw_in_own_draw_step(q, d.player))
}

/// "... that player discards a card instead. If the player discards a card this way, they draw a card. If the player
/// doesn't discard a card this way, they mill a card." The player chooses the card (CR 701.9b). The card drawn meets every
/// draw replacement effect but this one (CR 614.5): a second Chains makes them discard again (its ruling). With an empty
/// hand nothing is discarded, and the mill replaces the draw, so nothing is drawn at all.
fn discard_instead(g: &mut dyn Game, asker: &Asker, before: &Proposal, _: &Proposal) {
    if let Proposal::Draw(d) = before {
        let _ = discard_then_draw_or_mill(g, d.player, asker.source);
    }
}

fn discard_then_draw_or_mill(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<(), Suspend> {
    let chosen = choose_from_hand(g, player, source, 1)?;
    if g.discard_together(&chosen).is_empty() {
        g.mill(player, 1);
    } else {
        g.draw(player, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(
        ReplacementKind::Draw,
        a_player_would_draw,
        instead_of_drawing,
        "discards a card instead, then draws; mills a card if no card is discarded",
    )
    .then(discard_instead)],
    ..oracle::CHAINS_OF_MEPHISTOPHELES
};
