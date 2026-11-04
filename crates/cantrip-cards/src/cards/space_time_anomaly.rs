use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target player mills cards equal to your life total" (CR 701.17a): the spell's controller's life total as it resolves (not the
/// target's, CR 119.1), milled by the target player if they are still a legal target (CR 608.2b). A life total below zero (a player
/// Platinum Angel keeps in the game) mills no card: a negative count is zero (CR 107.1b).
fn mill_your_life_total(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        let n = g.state().player(item.controller).life.max(0) as u32;
        g.mill(player, n);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], mill_your_life_total),
    ..oracle::SPACE_TIME_ANOMALY
};
