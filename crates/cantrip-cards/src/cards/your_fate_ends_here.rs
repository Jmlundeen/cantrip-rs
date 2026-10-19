use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker with mana value 3 or greater" (CR 115.1, 202.3): read from the permanent's mana
/// value as it is targeted and checked again as the spell resolves (CR 608.2b).
fn mana_value_three_or_greater(q: &dyn Query, me: &Asker, t: Target) -> bool {
    (targets::CREATURE_OR_PLANESWALKER.filter)(q, me, t)
        && matches!(t, Target::Object(id) if q.mana_value(id) >= 3)
}

fn destroy_and_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    effects::surveil(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(mana_value_three_or_greater)], destroy_and_surveil),
    ..oracle::YOUR_FATE_ENDS_HERE
};
