use cantrip_api::*;
use cantrip_mechanics::triggers::cycling;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets -1/-1 until end of turn."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -1, -1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    abilities: &[cycling(mana("{B}"), "Cycling {B} ({B}, Discard this card: Draw a card.)")],
    ..oracle::LOCUST_SPRAY
};
