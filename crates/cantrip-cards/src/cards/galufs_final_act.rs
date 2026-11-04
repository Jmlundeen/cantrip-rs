use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Until end of turn, target creature gets +1/+0 and gains 'When this creature dies, ...'" (CR 611.2a, 613.1f): one instruction,
/// the creature being the spell's target; nothing if it is no longer a legal target as the spell resolves (CR 608.2b). The
/// granted ability is the card's delayed ability 0, a dies trigger of the creature's own until the cleanup step.
fn pump_and_gain_the_dies_trigger(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let this = g.card_id(&CARD);
    instruction(g, |g| {
        cantrip_mechanics::pump(g, item.object, creature, 1, 0);
        g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
    });
    Ok(())
}

/// "When this creature dies, put a number of +1/+1 counters equal to its power on up to one target creature." (CR 603.10a, 608.2h):
/// its power as it last existed on the battlefield, the +1/+0 included, none for a power of 0 or less (CR 107.1b). Nothing if no
/// target was chosen or it is no longer legal (CR 608.2b).
fn counters_equal_to_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let power = targets::power_or_last_known(g, item.object);
    g.add_counters(target, CounterKind::PlusOne, power.max(0));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], pump_and_gain_the_dies_trigger),
    delayed: &[triggered(
        THIS_DIES,
        counters_equal_to_its_power,
        "When this creature dies, put a number of +1/+1 counters equal to its power on up to one target creature.",
    )
    .targets(&[targets::CREATURE.optional()])],
    ..oracle::GALUFS_FINAL_ACT
};
