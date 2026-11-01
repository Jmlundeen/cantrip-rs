use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, lands};
use cantrip_oracle as oracle;

/// "You draw two cards and lose 2 life." (CR 121.1, 118.4): both happen to the spell's controller, in that order.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), own_cost_changes: &[cost_reduction::less_if_you_control_subtype::<1, { lands::sub(Subtype::Villain) }>], ..oracle::VISIONS_OF_VILLAINY };
