use cantrip_api::*;
use cantrip_mechanics::statics::entering;
use cantrip_oracle as oracle;

/// "If a Wizard you control entering causes a triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time." (CR 603.2d, 603.6a): the entering permanent judged as it is on the battlefield, continuous
/// effects applied (the rulings), and under Naban's controller; Naban's own entering counts.
fn wizard_you_control_entering(q: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    triggering.of_a_permanent_controlled_by(asker.controller)
        && entering(event).is_some_and(|permanent| {
            q.controller(permanent) == asker.controller && q.chars(permanent).has_subtype(q.words(asker.source).subtype(Subtype::Wizard))
        })
}

pub static CARD: CardDef = CardDef { additional_triggers: &[wizard_you_control_entering], ..oracle::NABAN_DEAN_OF_ITERATION };
