use cantrip_api::*;
use cantrip_mechanics::triggers::PROWESS;
use cantrip_oracle as oracle;

/// "If a triggered ability of a Shaman or another Wizard you control triggers, that ability triggers an additional time."
/// (CR 603.2d): the permanent as it is when the ability triggers; the Prodigy's own abilities only while it is a Shaman. The
/// creature types are words of its text (CR 612).
fn shaman_or_another_wizard(q: &dyn Query, asker: &Asker, _: &Event, triggering: &Triggering) -> bool {
    let words = q.words(asker.source);
    let c = q.chars(triggering.source);
    triggering.of_a_permanent_controlled_by(asker.controller)
        && (c.has_subtype(words.subtype(Subtype::Shaman))
            || (triggering.source != asker.source && c.has_subtype(words.subtype(Subtype::Wizard))))
}

pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    additional_triggers: &[shaman_or_another_wizard],
    ..oracle::HARMONIC_PRODIGY
};
