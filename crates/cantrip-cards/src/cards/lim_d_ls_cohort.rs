use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "... that creature can't be regenerated this turn." (CR 608.2, 611.2a): the other creature in the block, the trigger's subject.
fn creature_cant_be_regenerated(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(other) = item.subject.object() else { return Ok(()) };
    instruction(g, |g| g.add_effect(item.object, other, EffectKind::CantBeRegenerated, Duration::EndOfTurn));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::BLOCKS, triggers::blocks_or_becomes_blocked),
        creature_cant_be_regenerated,
        "Whenever this creature blocks or becomes blocked by a creature, that creature can't be regenerated this turn.",
    )],
    ..oracle::LIM_D_LS_COHORT
};
