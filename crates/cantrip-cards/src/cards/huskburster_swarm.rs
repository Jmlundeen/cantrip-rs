use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each creature card you own in exile and in your graveyard" (CR 601.2f, 118.7a):
/// counted as the cost is determined. The zones of a player hold the cards they own (CR 404.1, 406.1), whoever controlled
/// them last. Only the cost of casting it.
fn less_per_creature_card_you_own_in_exile_and_graveyard(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let spell = matches!(costed, Costed::Spell(_));
    let cards = [Zone::graveyard(player), Zone::exile(player)]
        .into_iter()
        .flat_map(|zone| q.state().zone(zone).iter())
        .filter(|&&id| spell && q.chars(id).is(CardType::Creature))
        .count();
    CostDelta::less(cards.min(255) as u8)
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_per_creature_card_you_own_in_exile_and_graveyard],
    ..oracle::HUSKBURSTER_SWARM
};
