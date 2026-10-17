use cantrip_api::*;
use cantrip_mechanics::effects::pump_all_creatures;
use cantrip_oracle as oracle;

/// All creatures get -X/-X until end of turn, for the X paid in life as an additional cost.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as i16;
    pump_all_creatures(g, item.object, -x, -x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, pay X life.
    additional_cost: Some(Cost { pay_x_life: true, ..Cost::NONE }),
    ..oracle::TOXIC_DELUGE
};
