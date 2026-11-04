use cantrip_api::*;
use cantrip_mechanics::{damage_target, lands, targets};
use cantrip_oracle as oracle;

/// "It deals 3 damage to any target. Target player draws three cards and gains 3 life." (CR 120.3, 121.1, 119.3): the two targets
/// are chosen as the ability is activated and each is judged alone as it resolves (CR 608.2b), so one gone does not stop the other
/// half. The Mechan is already sacrificed, so it deals the damage as it last existed (CR 113.7a, 608.2h). The player draws, then
/// gains the life, even when the library was too short to draw all three.
fn damage_then_target_player_draws_and_gains_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    if let Some(player) = targets::player(g, item, 1) {
        g.draw(player, 3);
        g.gain_life(player, 3);
    }
    Ok(())
}

/// "This ability costs {X} less to activate, where X is the number of differently named lands you control" (CR 601.2f): counted
/// once, as the cost is determined, before any mana abilities are activated to pay it (the 2025-07-25 ruling).
fn cost_reduction(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(lands::differently_named_lands(q, asker.controller) as u32)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{10}").and(SACRIFICE_THIS),
        damage_then_target_player_draws_and_gains_life,
        "{10}, Sacrifice this creature: It deals 3 damage to any target. Target player draws three cards and gains 3 life. This ability costs {X} less to activate, where X is the number of differently named lands you control.",
    )
    .targets(&[targets::ANY, targets::PLAYER])
    .cost_change(cost_reduction)],
    ..oracle::SURVEY_MECHAN
};
