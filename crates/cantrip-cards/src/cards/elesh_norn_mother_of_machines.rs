use cantrip_api::*;
use cantrip_mechanics::statics::{entering, permanent_entering_triggers_again};
use cantrip_oracle as oracle;

/// "Permanents entering don't cause abilities of permanents your opponents control to trigger." (CR 603.2, 603.6a): an
/// event that puts a permanent onto the battlefield, whoever controls it, triggers no ability of a permanent an opponent
/// controls. An opponent's ability that is no permanent's (a card's in a graveyard, an emblem's, a delayed one) is left alone.
fn opponents_permanents_dont_trigger(q: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    entering(event).is_some() && triggering.permanent && q.state().opponents(asker.controller).any(|p| p == triggering.controller)
}

/// "If a permanent entering causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time." (CR 603.2d)
pub static CARD: CardDef = CardDef {
    trigger_rules: &[opponents_permanents_dont_trigger],
    additional_triggers: &[permanent_entering_triggers_again],
    ..oracle::ELESH_NORN_MOTHER_OF_MACHINES
};
