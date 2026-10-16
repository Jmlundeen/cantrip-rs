use cantrip_api::*;
use cantrip_oracle as oracle;

/// "End the turn." (CR 724.1): Time Stop is exiled from the stack with every other spell and ability on it (724.1b).
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    g.end_the_turn()
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TIME_STOP };
