use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If a triggered ability of another Elemental you control triggers, it triggers an additional time." (CR 603.2d): the
/// ability's source a permanent other than this one, an Elemental under this one's controller as it is when the ability
/// triggers (as it last existed for one that looks back).
fn another_elemental_you_control(q: &dyn Query, asker: &Asker, _: &Event, triggering: &Triggering) -> bool {
    triggering.of_a_permanent_controlled_by(asker.controller)
        && triggering.source != asker.source
        && q.chars(triggering.source).has_subtype(q.words(asker.source).subtype(Subtype::Elemental))
}

pub static CARD: CardDef = CardDef { additional_triggers: &[another_elemental_you_control], ..oracle::TWINFLAME_TRAVELERS };
