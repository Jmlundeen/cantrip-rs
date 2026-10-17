use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker with mana value 3 or less" (CR 202.3).
fn creature_or_planeswalker_mv3(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_OR_PLANESWALKER.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// "Destroy target creature or planeswalker with mana value 3 or less." Nothing happens if it is no longer a legal target
/// (CR 608.2b). "This spell can't be countered" is the card's keyword (CR 101.2), ward's counter included.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_or_planeswalker_mv3)], effect),
    ..oracle::LONG_GOODBYE
};
