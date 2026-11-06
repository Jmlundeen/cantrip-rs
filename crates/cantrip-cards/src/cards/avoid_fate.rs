use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// An instant or Aura spell currently targeting a permanent you control (CR 115.1, 608.2b).
fn eligible_spell(q: &dyn Query, asker: &Asker, target: Target) -> bool {
    let Target::Object(id) = target else { return false };
    let Some(item) = q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == id) else { return false };
    let chars = q.chars(id);
    (chars.is(CardType::Instant) || chars.has_subtype(q.words(asker.source).subtype(Subtype::Aura)))
        && targets::spell_targets_permanent_controlled_by(q, item, asker.controller)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(eligible_spell)], effects::counter_first_target),
    ..oracle::AVOID_FATE
};
