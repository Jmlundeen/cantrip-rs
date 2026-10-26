use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+0 until end of turn. When that creature dies this turn, you draw a card." The delayed
/// triggered ability (CR 603.7c) is fixed on that creature, so it triggers only when that very object dies (a creature
/// that left and came back is a new object, CR 400.7).
fn a_good_day_to_die(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::pump_target_then_when_it_dies(g, item, 2, 0);
    Ok(())
}

/// "You draw a card", by the spell's controller.
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], a_good_day_to_die),
    delayed: &[triggered(triggers::THIS_DIES, draw, "When that creature dies this turn, you draw a card.")],
    ..oracle::A_GOOD_DAY_TO_DIE
};
