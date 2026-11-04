use cantrip_api::*;
use cantrip_mechanics::creatures_you_control;
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if your opponents control three or more creatures" (CR 601.2f, 118.7): `player` is the one
/// casting it, the creatures counted are those all their opponents control together, and only generic mana is reduced. It changes
/// only the total cost: the mana value stays 6.
fn cheaper_against_three_creatures(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    let creatures: usize = q.state().opponents(player).map(|opponent| creatures_you_control(q, opponent)).sum();
    if creatures >= 3 { CostDelta::less(2) } else { CostDelta::NONE }
}

pub static CARD: CardDef = CardDef { own_cost_changes: &[cheaper_against_three_creatures], ..oracle::LASHWHIP_PREDATOR };
