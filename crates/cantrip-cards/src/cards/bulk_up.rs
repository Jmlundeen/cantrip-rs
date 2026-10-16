use cantrip_api::*;
use cantrip_mechanics::{double, flashback, targets};
use cantrip_oracle as oracle;

/// "Double target creature's power until end of turn." (CR 701.10b-c, shared in `double::power`.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        double::power(g, item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ways: &[flashback::way(cost("{4}{R}{R}"), "Flashback {4}{R}{R}")],
    ..oracle::BULK_UP
};
