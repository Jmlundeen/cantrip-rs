use cantrip_api::*;
use cantrip_mechanics::{pump, targets, warp};
use cantrip_oracle as oracle;

/// "Target creature you control gets +3/+2 until end of turn. If that creature was cast for its warp cost, it also gains
/// trample and haste until end of turn." Whether it was is read as the spell resolves, from the creature (CR 702.185a).
fn bore(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 3, 2);
    if warp::was_cast_for_warp(g, creature) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample, Keyword::Haste])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], bore),
    ..oracle::FULL_BORE
};
