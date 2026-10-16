use cantrip_api::*;
use cantrip_mechanics::{delayed, manifest_dread, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+0 until end of turn. When it dies this turn, manifest dread." The delayed triggered ability
/// (CR 603.7c) is fixed on that creature, which is its source, so it triggers only when that very object dies (a
/// creature that left and came back is a new object, CR 400.7).
fn turn_inside_out(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::pump(g, item.object, creature, 3, 0);
    delayed::this_turn(g, item, creature);
    Ok(())
}

/// "Manifest dread" (CR 701.62a), by the spell's controller.
fn manifest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    manifest_dread::manifest_dread(g, item.controller, item.object)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], turn_inside_out),
    delayed: &[triggered(triggers::THIS_DIES, manifest, "When it dies this turn, manifest dread.").look_back()],
    ..oracle::TURN_INSIDE_OUT
};
