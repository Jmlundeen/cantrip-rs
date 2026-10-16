use cantrip_api::*;
use cantrip_mechanics::statics::entering;
use cantrip_oracle as oracle;

/// "Artifacts and creatures entering don't cause abilities to trigger." (CR 603.2, 603.6a): an event that puts an artifact
/// or a creature onto the battlefield, judged as it is there, triggers no ability. A land that is an artifact or a creature
/// too stops a landfall ability; one that is neither does not (the rulings' Tireless Tracker).
fn artifacts_and_creatures_entering(q: &dyn Query, _: &Asker, event: &Event, _: &Triggering) -> bool {
    entering(event).is_some_and(|permanent| {
        let chars = q.chars(permanent);
        chars.is(CardType::Artifact) || chars.is(CardType::Creature)
    })
}

pub static CARD: CardDef = CardDef { trigger_rules: &[artifacts_and_creatures_entering], ..oracle::DOORKEEPER_THRULL };
