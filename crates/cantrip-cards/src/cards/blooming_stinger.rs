use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Another target creature you control gains deathtouch until end of turn" (CR 611.2a, 702.2), if it is still a legal target as
/// this resolves (CR 608.2b).
fn gain_deathtouch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Deathtouch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        gain_deathtouch,
        "When this creature enters, another target creature you control gains deathtouch until end of turn.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::BLOOMING_STINGER
};
