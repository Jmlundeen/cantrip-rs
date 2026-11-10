use cantrip_api::*;
use cantrip_mechanics::{effects, flashback};
use cantrip_oracle as oracle;

/// "Attacking creatures get +2/+0 until end of turn" (CR 611.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::attacking_creatures_get(g, item.object, 2, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{2}{W}{W}"), "Flashback {2}{W}{W}")],
    ..oracle::AURONS_INSPIRATION
};
