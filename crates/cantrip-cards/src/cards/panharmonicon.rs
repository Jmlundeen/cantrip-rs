use cantrip_api::*;
use cantrip_mechanics::statics::entering;
use cantrip_oracle as oracle;

/// "If an artifact or creature entering causes a triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time." (CR 603.2d, 603.6a): the entering permanent judged as it is on the battlefield, whoever
/// controls it (the rulings).
fn artifact_or_creature_entering(q: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    triggering.of_a_permanent_controlled_by(asker.controller)
        && entering(event).is_some_and(|permanent| {
            let c = q.chars(permanent);
            c.is(CardType::Artifact) || c.is(CardType::Creature)
        })
}

pub static CARD: CardDef = CardDef { additional_triggers: &[artifact_or_creature_entering], ..oracle::PANHARMONICON };
