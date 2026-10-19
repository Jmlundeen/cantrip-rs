use cantrip_api::*;
use cantrip_mechanics::triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS;
use cantrip_oracle as oracle;

/// "Arni gets +X/+0 until end of turn, where X is that creature's power" (CR 603.6a, 208.1): the entering creature is the
/// trigger's subject, read as it entered.
fn pump_by_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(entering) = item.subject {
        let power = g.chars(entering).power.unwrap_or(0) as i16;
        cantrip_mechanics::pump(g, item.object, item.object, power, 0);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(ANOTHER_CREATURE_YOU_CONTROL_ENTERS, pump_by_power, "Whenever another creature you control enters, Arni gets +X/+0 until end of turn, where X is that creature's power.")],
    ..oracle::ARNI_RENOWNED_CHAMPION
};
