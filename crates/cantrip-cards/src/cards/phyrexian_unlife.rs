use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You don't lose the game for having 0 or less life" (CR 104.3b, 704.5a): only that way; ten poison counters or a draw
/// from an empty library still lose (the 2011-06-01 ruling).
fn no_loss_for_life(_: &dyn Query, asker: &Asker, player: PlayerId, reason: LossReason) -> bool {
    player == asker.controller && reason == LossReason::Life
}

/// "As long as you have 0 or less life, all damage is dealt to you as though its source had infect" (CR 702.90b): asked as
/// the damage is dealt, so damage that takes a positive life total to 0 or less is dealt as life loss (the 2011-06-01 ruling).
fn infect_at_zero_or_less(q: &dyn Query, asker: &Asker, player: PlayerId) -> bool {
    player == asker.controller && q.state().player(player).life <= 0
}

pub static CARD: CardDef = CardDef {
    loss_rules: &[no_loss_for_life],
    infect_damage_to: &[infect_at_zero_or_less],
    ..oracle::PHYREXIAN_UNLIFE
};
