use cantrip_api::*;
use cantrip_mechanics::{history, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, this creature deals 1 damage to each opponent and planeswalker it has dealt damage to this
/// game." (CR 120.3, 603.2): read from the event history, and dealt together (CR 120.2).
fn deal_1_damage_to_each_it_has_damaged(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players = history::players_dealt_damage_by(g.history(), item.object, 0);
    let planeswalkers = history::permanents_dealt_damage_by(g.history(), item.object, 0);
    let batch: Vec<(ObjectId, Target, u32)> = g
        .state()
        .opponents(item.controller)
        .filter(|p| players.contains(p))
        .map(Target::Player)
        .chain(
            g.state()
                .battlefield()
                .iter()
                .copied()
                .filter(|&id| planeswalkers.contains(&id) && g.chars(id).is(CardType::Planeswalker))
                .map(Target::Object),
        )
        .map(|t| (item.object, t, 1))
        .collect();
    g.damage_together(&batch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_UPKEEP,
        deal_1_damage_to_each_it_has_damaged,
        "At the beginning of your upkeep, this creature deals 1 damage to each opponent and planeswalker it has dealt damage to this game.",
    )],
    ..oracle::THE_FALLEN
};
