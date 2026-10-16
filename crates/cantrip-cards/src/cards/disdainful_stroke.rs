use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Target spell with mana value 4 or greater" (CR 115.1, 202.3): a spell on the stack whose mana value is asked of the
/// spell as it is (an {X} counts as the value chosen for it, CR 202.3e, and a face-down spell is 0, CR 708.2). Asked as
/// the spell is cast and again as it resolves (CR 608.2b).
fn big_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) >= 4)
}

/// Counter target spell with mana value 4 or greater (CR 701.6). A spell whose target became illegal is not resolved
/// (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(big_spell)], effect),
    ..oracle::DISDAINFUL_STROKE
};
