use cantrip_api::*;
use cantrip_mechanics::{effects, learn, targets};
use cantrip_oracle as oracle;

/// "Target spell or permanent with mana value 1 or greater" (CR 115.1, 202.3): a spell's mana value counts an {X} as the value
/// chosen for it (CR 202.3e); a permanent's counts it as 0, and a token with no mana cost is 0. Asked as the spell is cast
/// and again as it resolves (CR 608.2b).
fn spell_or_permanent_with_mana_value_at_least_one(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL_OR_PERMANENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) >= 1)
}

/// "Return target spell or permanent with mana value 1 or greater to its owner's hand. Learn." A spell leaves the stack for
/// its owner's hand without being countered, a token ceases to exist (CR 111.7); Learn (CR 701.48a) follows as its own sentence.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(object) = targets::object(g, item, 0) {
        effects::return_spell_or_permanent_to_hand(g, object);
    }
    g.end_sentence();
    learn::learn(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(spell_or_permanent_with_mana_value_at_least_one)], effect),
    ..oracle::DIVIDE_BY_ZERO
};
