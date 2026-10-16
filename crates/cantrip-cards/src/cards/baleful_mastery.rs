use cantrip_api::*;
use cantrip_mechanics::{choose, exile_target, targets};
use cantrip_oracle as oracle;

/// If the {1}{B} cost was paid, an opponent draws a card; then exile the target. "An opponent" is one the controller
/// chooses as the spell resolves, asked only when there are several.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.way == 1 {
        let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
        let drawer = match opponents.len() {
            0 => None,
            1 => Some(opponents[0]),
            _ => choose::player(g, item.controller, item.object, opponents, "Draw", 1)?,
        };
        if let Some(player) = drawer {
            g.draw(player, 1);
        }
    }
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ways: &[way(cost("{1}{B}"), ZoneKind::Hand, "Pay {1}{B}; an opponent draws a card")],
    ..oracle::BALEFUL_MASTERY
};
