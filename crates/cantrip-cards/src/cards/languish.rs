use cantrip_api::*;
use cantrip_mechanics::effects::pump_all_creatures;
use cantrip_oracle as oracle;

/// "All creatures get -4/-4 until end of turn." The creatures of every player are affected, the set fixed as the spell
/// resolves (CR 611.2c); those left with toughness 0 or less die as a state-based action (CR 704.5f).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump_all_creatures(g, item.object, -4, -4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::LANGUISH
};
