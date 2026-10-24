use cantrip_api::*;
use cantrip_mechanics::{each_discards_a_card, hand, targets, triggers};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each creature card in your graveyard" (CR 601.2f, 118.7a): counted as the cost is
/// determined, the cards as they are in the graveyard. Only the cost of casting it. Its mana value stays 7 (CR 118.7).
fn less_per_creature_card_in_your_graveyard(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let spell = matches!(costed, Costed::Spell(_));
    let cards = q.state().zone(Zone::graveyard(player)).iter().filter(|&&id| spell && q.chars(id).is(CardType::Creature)).count();
    CostDelta::less(cards.min(255) as u8)
}

/// "When this creature enters, any number of target opponents each discard a card. For each of those opponents who didn't
/// discard a card with mana value 4 or greater, draw a card." Each target still legal as it resolves (CR 608.2b) chooses a
/// card in turn order from the active player and they are discarded together (CR 101.4, 701.9); an opponent with no card
/// in hand discarded none, so you draw for them.
fn discard_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let legal = targets::legal_players(g, item);
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|p| legal.contains(p)).collect();
    let discarded = each_discards_a_card(g, &opponents, item.object)?;
    // The cards are in graveyards now, so their mana values are those of the cards (CR 202.3); one a replacement put into a
    // hidden zone unrevealed is undefined and has none (CR 701.9c, `hand::discarded_chars`).
    let big = |g: &dyn Game, p: PlayerId| {
        discarded.iter().any(|&c| g.state().obj(c).owner == p && hand::discarded_chars(g, c).is_some() && g.mana_value(c) >= 4)
    };
    let draws = opponents.iter().filter(|&&p| !big(g, p)).count();
    g.draw(item.controller, draws as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_per_creature_card_in_your_graveyard],
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        discard_then_draw,
        "When this creature enters, any number of target opponents each discard a card. For each of those opponents who didn't discard a card with mana value 4 or greater, draw a card.",
    )
    .targets(&[targets::OPPONENT.between(0, TargetSpec::ANY_NUMBER).labelled("Opponents who each discard a card")])],
    ..oracle::HOLLOW_MARAUDER
};
