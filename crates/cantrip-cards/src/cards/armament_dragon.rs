use cantrip_api::*;
use cantrip_mechanics::{counters_divided, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Distribute three +1/+1 counters among one, two, or three target creatures you control" (CR 601.2d, 603.3d): the targets and
/// their shares are locked in as the ability is put on the stack, one counter at least on each; a target that has become illegal
/// since is taken out with its share, so those counters are lost (CR 608.2b).
fn distribute_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counters_divided(g, item, CounterKind::PlusOne);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        distribute_three,
        "When this creature enters, distribute three +1/+1 counters among one, two, or three target creatures you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL.between(1, 3)])
    .divide(Divide { total: 3, reads: None })],
    ..oracle::ARMAMENT_DRAGON
};
