use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Blocking creatures get +0/+3 until end of turn." (CR 608.2, 611.2a).
fn blocking_creatures_get_0_3(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::blocking_creatures_get(g, item.object, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], blocking_creatures_get_0_3),
    ..oracle::PIETY
};
