use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// A spell whose mana value is the X announced for Spell Blast (CR 107.3a, 202.3e).
fn spell_with_mana_value_x(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t) && t.object().is_some_and(|id| targets::mana_value_is_x(q, id))
}

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(spell_with_mana_value_x)], counter),
    late_targets: true,
    ..oracle::SPELL_BLAST
};
