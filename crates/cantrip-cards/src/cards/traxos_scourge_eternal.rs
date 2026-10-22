use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast an artifact or creature spell" (CR 601.2i, 603.2c): one trigger for either type, including a spell
/// that is both, and only its controller's casts, not copies or permanents entering.
fn artifact_or_creature_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::artifact_spell_you_cast(q, me, event).or_else(|| triggers::creature_spell_you_cast(q, me, event))
}

pub static CARD: CardDef = CardDef {
    // "Traxos doesn't untap during your untap step" (CR 502.3), not an enters-tapped ability.
    keywords: Keywords::of(&[Keyword::Trample, Keyword::DoesNotUntap]),
    abilities: &[
        triggered(when_asked(EventMask::SPELL_CAST, artifact_or_creature_spell_you_cast), effects::untap_self, "Whenever you cast an artifact or creature spell, untap Traxos."),
    ],
    ..oracle::TRAXOS_SCOURGE_ETERNAL
};
