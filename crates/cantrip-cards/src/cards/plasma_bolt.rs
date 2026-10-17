use cantrip_api::*;
use cantrip_mechanics::{damage_target, history, targets};
use cantrip_oracle as oracle;

/// "Plasma Bolt deals 2 damage to any target. Void - Plasma Bolt deals 3 damage instead if a nonland permanent left the
/// battlefield this turn or a spell was warped this turn": the void condition is read as the spell resolves (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let amount = if history::void(g) { 3 } else { 2 };
    damage_target(g, item, 0, amount);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ANY], effect), ..oracle::PLASMA_BOLT };
