use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Attacking creatures get +2/+0 until end of turn." (CR 611.2c).
fn pump_attackers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::attacking_creatures_get(g, item.object, 2, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], pump_attackers), ..oracle::ARMY_OF_ALLAH };
