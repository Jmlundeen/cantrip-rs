use cantrip_api::*;
use cantrip_mechanics::{damage_target, history, targets};
use cantrip_oracle as oracle;

/// "Fling deals damage equal to the sacrificed creature's power to any target": its power as it last existed on the
/// battlefield (CR 608.2h, the ruling); a power of 0 or less deals no damage (CR 120.8).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = history::sacrificed_creatures_power(g, item.object);
    if power > 0 {
        damage_target(g, item, 0, power.min(u16::MAX as i32) as u16);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8, 601.2h): exactly one.
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    ..oracle::FLING
};
