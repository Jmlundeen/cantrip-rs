use cantrip_api::*;
use cantrip_mechanics::triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS;
use cantrip_oracle as oracle;

/// "Arni gets +X/+0 until end of turn, where X is that creature's power" (CR 603.6a, 208.1): the entering creature is the
/// trigger's subject, its power read as the ability resolves (CR 608.2h). A negative power gives +0/+0 (CR 107.1b).
fn pump_by_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(entering) = item.subject {
        let power = g.chars(entering).power.unwrap_or(0).max(0);
        cantrip_mechanics::pump(g, item.object, item.object, power, 0);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(ANOTHER_CREATURE_YOU_CONTROL_ENTERS, pump_by_power, "Whenever another creature you control enters, Arni gets +X/+0 until end of turn, where X is that creature's power.")],
    ..oracle::ARNI_RENOWNED_CHAMPION
};
