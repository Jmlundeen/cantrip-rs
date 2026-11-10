use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{1}, {T}: You and target opponent each draw two cards." The opponent is chosen as the ability is activated and must still be
/// a legal target as it resolves (CR 115.1, 608.2b); the cards are drawn one player at a time, this player first, as the text
/// names them (CR 121.2a, 101.4).
fn you_and_target_opponent_each_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    g.draw(item.controller, 2);
    g.draw(opponent, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(TAP), you_and_target_opponent_each_draw_two,
        "{1}, {T}: You and target opponent each draw two cards.")
        .targets(&[targets::OPPONENT])],
    ..oracle::COMMANDER_WILLIAM_RIKER
};
