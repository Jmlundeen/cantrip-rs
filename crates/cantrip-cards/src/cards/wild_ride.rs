use cantrip_api::*;
use cantrip_mechanics::{harmonize, haste_until_end_of_turn, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+0 and gains haste until end of turn." Both are continuous effects of this spell (CR 611.2); if
/// the creature is no longer a legal target the spell does nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        instruction(g, |g| {
            pump(g, item.object, creature, 3, 0);
            haste_until_end_of_turn(g, item.object, creature);
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ways: &[harmonize::way(mana("{4}{R}"), "Harmonize {4}{R}")],
    ..oracle::WILD_RIDE
};
