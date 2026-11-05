use cantrip_api::*;
use cantrip_mechanics::{pump, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature an opponent controls gets -2/-2 until end of turn and target creature you control gets +2/+2 until end of
/// turn." (CR 603.3d, 608.2b, 611.2a): each target is judged on its own as this resolves, so one that has become illegal does not
/// stop the other from taking its modifier; only if both are illegal does the ability not resolve, and that is the core's.
fn shrink_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(theirs) = targets::object(g, item, 0) {
        pump(g, item.object, theirs, -2, -2);
    }
    g.end_sentence();
    let Some(yours) = targets::object(g, item, 1) else { return Ok(()) };
    pump(g, item.object, yours, 2, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        shrink_and_grow,
        "When this creature enters, target creature an opponent controls gets -2/-2 until end of turn and target creature you control gets +2/+2 until end of turn.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS, targets::CREATURE_YOU_CONTROL])],
    ..oracle::GURMAG_RAKSHASA
};
