use cantrip_api::*;
use cantrip_mechanics::{double, triggers};
use cantrip_oracle as oracle;

/// "Double Tifa Lockhart's power until end of turn" (CR 701.10b-c): +X/+0 for the power it has as the ability resolves, once for
/// each land that entered. Nothing happens if it has left the battlefield (the effect skips an object that is gone).
fn double_own_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    double::power(g, item.object, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::LANDFALL, double_own_power, "Landfall \u{2014} Whenever a land you control enters, double Tifa Lockhart's power until end of turn.")],
    ..oracle::TIFA_LOCKHART
};
