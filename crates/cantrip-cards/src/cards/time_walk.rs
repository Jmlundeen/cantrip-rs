use cantrip_api::*;
use cantrip_oracle as oracle;

/// Take an extra turn after this one (CR 500.7).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TIME_WALK };
