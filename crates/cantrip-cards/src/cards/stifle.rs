use cantrip_api::*;
use cantrip_mechanics::{counter_target_ability, targets};
use cantrip_oracle as oracle;

/// Counter target activated or triggered ability (CR 701.6, 115.1); a mana ability never is on the stack to be targeted
/// (CR 605.3b), nor is a special action or turn-based action.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target_ability(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ACTIVATED_OR_TRIGGERED_ABILITY], effect), ..oracle::STIFLE };
