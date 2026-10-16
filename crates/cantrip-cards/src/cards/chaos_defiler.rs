use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Battle Cannon: for each opponent, its controller chooses a nonland permanent that player controls (nobody is asked
/// about one candidate), then one of the chosen is destroyed at random (nothing is drawn for a single one).
fn battle_cannon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let opponents: Vec<PlayerId> = g.state().opponents(you).collect();
    let mut chosen: Vec<ObjectId> = Vec::new();
    for opponent in opponents {
        let theirs: Vec<ObjectId> =
            g.permanents_of(opponent).into_iter().filter(|&id| !g.chars(id).is(CardType::Land)).collect();
        if theirs.len() <= 1 {
            chosen.extend(theirs);
            continue;
        }
        chosen.extend(choose::card(g, you, item.object, Purpose::Destroy, theirs)?);
    }
    if let Some(victim) = cantrip_mechanics::library::random_of(g, &chosen) {
        g.destroy(victim);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, battle_cannon, "Battle Cannon - When Chaos Defiler enters, for each opponent, choose a nonland permanent that player controls. Destroy one of them chosen at random."),
        triggered(cantrip_mechanics::triggers::THIS_DIES, battle_cannon, "Battle Cannon - When Chaos Defiler dies, for each opponent, choose a nonland permanent that player controls. Destroy one of them chosen at random.")
            .look_back(),
    ],
    ..oracle::CHAOS_DEFILER
};
