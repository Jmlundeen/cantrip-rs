use cantrip_api::*;
use cantrip_mechanics::{damage_target, discard_chosen, targets};
use cantrip_oracle as oracle;

/// X damage to any target, then draw two cards and discard a card (the discard is the caster's choice).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, item.x);
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ANY], effect), ..oracle::TRAUMATIC_CRITIQUE };
