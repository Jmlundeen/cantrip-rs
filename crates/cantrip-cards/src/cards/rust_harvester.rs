use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, targets::filters};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature, then it deals damage equal to its power to any target." The counter only goes on a
/// Harvester still on the battlefield (CR 400.7); the damage is its power as the layers now make it, or as it last was there if it
/// has left (CR 608.2h), dealt by the Harvester as the source; nothing is dealt for a power of zero or less (CR 120.8) or to a
/// target that is no longer legal (CR 608.2b).
fn grow_then_deal_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u16;
    damage_target(g, item, 0, power);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}, {T}").and(exile_from_graveyard_of(1, filters::artifact)),
        grow_then_deal_its_power,
        "{2}, {T}, Exile an artifact card from your graveyard: Put a +1/+1 counter on this creature, then it deals damage equal to its power to any target.",
    )
    .targets(&[targets::ANY])],
    ..oracle::RUST_HARVESTER
};
