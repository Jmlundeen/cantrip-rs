use cantrip_api::*;
use cantrip_mechanics::delayed::{LOSE_AT_THAT_END_STEP, extra_turn_then_lose};
use cantrip_oracle as oracle;

/// "Take an extra turn after this one. At the beginning of that turn's end step, you lose the game." (CR 500.7, 603.7).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    extra_turn_then_lose(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), delayed: &[LOSE_AT_THAT_END_STEP], ..oracle::LAST_CHANCE };
