use cantrip_api::*;
use cantrip_mechanics::{goad, targets, triggers};
use cantrip_oracle as oracle;

/// "the number of opponents you attacked this turn" (CR 508.1, 603.3d): each player other than the ability's controller that
/// creature the ability's controller declared attacking this turn, counted once however many creatures attacked them; a
/// planeswalker or battle attacked is no player, and a creature put onto the battlefield attacking was never declared (CR 508.4).
fn opponents_attacked_this_turn(q: &dyn Query, asker: &Asker) -> u32 {
    let mut attacked: Vec<PlayerId> = Vec::new();
    for r in q.history().this_turn() {
        if let Event::Attacks { defender: Target::Player(p), .. } = r.event
            && r.active == asker.controller
            && p != asker.controller
            && !attacked.contains(&p)
        {
            attacked.push(p);
        }
    }
    attacked.len() as u32
}

/// "Goad up to X target creatures" (CR 701.15a): each target still a creature as the trigger resolves, goaded by the
/// ability's controller until that player's next turn.
fn goad_targets(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = targets::legal_objects(g, item);
    g.one_instruction(&mut |g| {
        for &creature in &creatures {
            goad::goad(g, item, creature);
        }
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        goad_targets,
        "Battlefront \u{2014} Whenever this creature attacks, goad up to X target creatures, where X is the number of opponents you attacked this turn.",
    )
    .targets(&[targets::CREATURE.between(0, TargetSpec::COUNTED)])
    .counted_by(opponents_attacked_this_turn)],
    ..oracle::GALVANIZING_TENOR
};
