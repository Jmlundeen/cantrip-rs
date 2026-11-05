use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ATTACKS};
use cantrip_oracle as oracle;

/// "Up to one target player mills cards equal to this creature's power" (CR 701.17a): the power it has as the ability resolves,
/// or the one it last had once it has left the battlefield (CR 608.2h); nothing when no player was chosen or the one chosen is no
/// longer a legal target (CR 608.2b).
fn target_player_mills_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        let power = targets::power_or_last_known(g, item.object).max(0) as u32;
        g.mill(player, power);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        target_player_mills_its_power,
        "Whenever this creature attacks, up to one target player mills cards equal to this creature's power.",
    )
    .targets(&[targets::PLAYER.optional()])],
    ..oracle::VETERAN_ICE_CLIMBER
};
