use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever you draw your second card each turn, double the number of each kind of counter on target artifact or creature
/// you control." The shared trigger counts your draws this turn (CR 121.1, 603.2). Doubling adds as many counters of every
/// kind as the legal target has when this resolves (CR 701.10e, 608.2b, 608.2h), all put on at once.
fn double_counters_on_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    double::each_kind_of_counter(g, &[target]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_DRAW_YOUR_SECOND_CARD,
        double_counters_on_target,
        "Whenever you draw your second card each turn, double the number of each kind of counter on target artifact or creature you control.",
    )
    .targets(&[targets::ARTIFACT_OR_CREATURE_YOU_CONTROL])],
    ..oracle::CHIEF_MILES_OBRIEN
};
