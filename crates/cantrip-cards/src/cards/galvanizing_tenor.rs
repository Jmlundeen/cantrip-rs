use cantrip_api::*;
use cantrip_mechanics::{battlefront, goad, targets, triggers};
use cantrip_oracle as oracle;

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
    .counted_by(|q, asker| battlefront::opponents_attacked_this_turn(q, asker.controller))],
    ..oracle::GALVANIZING_TENOR
};
