use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_BECOMES_TAPPED;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Another target creature you control gets +X/+X until end of turn, where X is this creature's power" (CR 611.2a, 608.2h): X is
/// worked out once, as the ability resolves (the ruling), from this creature's power as it last was if it has left the battlefield;
/// a negative power gives zero (CR 107.1b).
fn pump_by_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let x = g.chars(item.object).power.unwrap_or_default().max(0);
    pump(g, item.object, creature, x, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_BECOMES_TAPPED,
        pump_by_its_power,
        "Whenever this creature becomes tapped, another target creature you control gets +X/+X until end of turn, where X is this creature's power.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::HEMOSYMBIC_MITE
};
