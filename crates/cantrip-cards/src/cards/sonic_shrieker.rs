use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, history::players_dealt_damage_by, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "It deals 2 damage to any target and you gain 2 life. If a player is dealt damage this way, they discard a card." (CR 120.3,
/// 119.3, 701.9): each player the damage actually reached, read from the damage events this resolution recorded - not a target
/// a prevention kept it from (that records no damage), and whoever a redirection sent it to - discards a card of their choice after the damage and the life
/// gain are done.
fn strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let before = g.history().head() as usize;
    g.damage(item.object, item.target(0), 2);
    g.gain_life(item.controller, 2);
    let players = players_dealt_damage_by(g.history(), item.object, before);
    for p in players {
        discard_chosen(g, p, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        strike,
        "When this creature enters, it deals 2 damage to any target and you gain 2 life. If a player is dealt damage this way, they discard a card.",
    )
    .targets(&[targets::ANY])],
    ..oracle::SONIC_SHRIEKER
};
