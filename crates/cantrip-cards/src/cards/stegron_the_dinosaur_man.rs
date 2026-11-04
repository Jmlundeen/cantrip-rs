use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Until end of turn, target creature you control gets +3/+1 and becomes a Dinosaur in addition to its other types" (CR 611.2a,
/// 613.4b): both for the turn, as one instruction, to the creature the ability targeted; nothing if it is no longer a legal target
/// as this resolves (CR 608.2b).
fn pump_and_make_a_dinosaur(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        cantrip_mechanics::pump(g, item.object, creature, 3, 1);
        cantrip_mechanics::effects::becomes_subtype_until_end_of_turn(g, item.object, creature, Subtype::Dinosaur);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}{R}").and(DISCARD_THIS),
        pump_and_make_a_dinosaur,
        "Dinosaur Formula \u{2014} {1}{R}, Discard this card: Until end of turn, target creature you control gets +3/+1 and becomes a Dinosaur in addition to its other types.",
    )
    .from(ZoneKind::Hand)
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::STEGRON_THE_DINOSAUR_MAN
};
