use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {3} (CR 118.12). (Improvise is a characteristic of the card:
/// `pay_with`.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{3}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL], effect), ..oracle::METALLIC_REBUKE };
