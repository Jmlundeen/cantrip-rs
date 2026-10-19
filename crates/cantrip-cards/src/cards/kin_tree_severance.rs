use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Target permanent with mana value 3 or greater" (CR 115.1, 202.3: X in a mana cost is 0 on the battlefield, a copy has
/// its copiable mana value, a face-down permanent 0): any permanent, land or not, whoever controls it.
fn permanent_with_mana_value_3_or_greater(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.mana_value(id) >= 3)
}

/// "Exile target permanent with mana value 3 or greater."
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(permanent_with_mana_value_3_or_greater)], exile),
    ..oracle::KIN_TREE_SEVERANCE
};
