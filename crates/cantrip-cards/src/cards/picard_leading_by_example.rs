use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "{1}, {T}: Another target creature gets +X/+X until end of turn, where X is Picard's power." X is Picard's power as the
/// ability resolves (CR 608.2h), read before the pump, and 0 where that power is negative (CR 107.1b); "another" leaves
/// Picard out (CR 115.1d).
fn pump_by_picards_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let x = g.chars(item.object).power.unwrap_or(0).max(0);
    pump(g, item.object, creature, x, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}").and(TAP),
        pump_by_picards_power,
        "{1}, {T}: Another target creature gets +X/+X until end of turn, where X is Picard's power. Activate only as a sorcery.",
    )
    .targets(&[targets::OTHER_CREATURE])
    .sorcery_speed()],
    ..oracle::PICARD_LEADING_BY_EXAMPLE
};
