use cantrip_api::*;
use cantrip_mechanics::effects::{destroy_all_creatures, pump_all_creatures};
use cantrip_mechanics::history::gained_life_this_turn;
use cantrip_oracle as oracle;

/// "All creatures get -2/-2 until end of turn. Infusion \u{2014} If you gained life this turn, destroy all creatures
/// instead." Whether the controller gained life this turn is read as the spell resolves (CR 608.2c, 119.3), from the
/// event history.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if gained_life_this_turn(g, item.controller) {
        destroy_all_creatures(g, item)
    } else {
        pump_all_creatures(g, item.object, -2, -2);
        Ok(())
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::WITHERING_CURSE
};
