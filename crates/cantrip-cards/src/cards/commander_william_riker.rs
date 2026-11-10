use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{1}, {T}: You and target opponent each draw two cards." The opponent is chosen as the ability is activated and must still be
/// a legal target as it resolves (CR 115.1, 608.2b); the shared `you_and_target_player_each_draw` draws in turn order from the
/// active player (CR 121.2c), who is not always the controller.
fn you_and_target_opponent_each_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::you_and_target_player_each_draw(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(TAP), you_and_target_opponent_each_draw_two,
        "{1}, {T}: You and target opponent each draw two cards.")
        .targets(&[targets::OPPONENT])],
    ..oracle::COMMANDER_WILLIAM_RIKER
};
