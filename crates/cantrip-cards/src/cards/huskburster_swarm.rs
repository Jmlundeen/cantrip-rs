use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each creature card you own in exile and in your graveyard" (CR 601.2f, 118.7a):
/// counted as the cost is determined. The zones of a player hold the cards they own (CR 404.1, 406.1), whoever controlled
/// them last. Only the cost of casting it.
fn less_per_creature_card_you_own_in_exile_and_graveyard(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    let cards = [Zone::graveyard(player), Zone::exile(player)]
        .into_iter()
        .flat_map(|zone| q.state().zone(zone).iter())
        .filter(|&&id| q.chars(id).is(CardType::Creature))
        .count();
    CostDelta::less(cards as u32)
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_per_creature_card_you_own_in_exile_and_graveyard],
    ..oracle::HUSKBURSTER_SWARM
};
