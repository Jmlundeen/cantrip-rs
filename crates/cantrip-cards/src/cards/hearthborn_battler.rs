use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, triggers};
use cantrip_oracle as oracle;

/// "... this creature deals 2 damage to target opponent": the target is chosen when the ability goes on the stack, any
/// opponent of its controller (not necessarily the player who cast the spell, the ruling), and nothing happens if it is
/// no longer a legal target (CR 608.2b).
fn two_damage_to_target_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::A_PLAYER_CASTS_THEIR_SECOND_SPELL,
        two_damage_to_target_opponent,
        "Whenever a player casts their second spell each turn, this creature deals 2 damage to target opponent.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::HEARTHBORN_BATTLER
};
