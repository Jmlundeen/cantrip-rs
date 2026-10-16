use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Boltwave deals 3 damage to each opponent." (CR 120.3, 102.2): one damage event for each opponent still in the game,
/// dealt together.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::BOLTWAVE };
