use cantrip_api::*;
use cantrip_mechanics::effects::{gain_indestructible_until_end_of_turn, gain_lifelink_until_end_of_turn};
use cantrip_mechanics::{affinity, lands, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+0 and gains lifelink and indestructible until end of turn" (CR 611.2a): one layer 7c and two
/// layer 6 effects, from this spell, on the creature if it is still a legal target.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 2, 0);
        gain_lifelink_until_end_of_turn(g, item.object, creature);
        gain_indestructible_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    own_cost_changes: &[affinity::for_creature_type::<{ lands::sub(Subtype::Lizard) }>],
    ..oracle::SCALES_OF_SHALE
};
