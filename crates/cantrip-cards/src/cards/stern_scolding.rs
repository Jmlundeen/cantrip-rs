use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature spell with power or toughness 2 or less" (CR 115.1): a spell on the stack, read as it is there.
fn small_creature_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_SPELL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if {
            let c = q.chars(id);
            c.power.is_some_and(|p| p <= 2) || c.toughness.is_some_and(|t| t <= 2)
        })
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(small_creature_spell)], effects::counter_first_target),
    ..oracle::STERN_SCOLDING
};
