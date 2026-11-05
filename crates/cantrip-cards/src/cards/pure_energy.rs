use cantrip_api::*;
use cantrip_mechanics::{beam_me_up, counters_divided, targets};
use cantrip_oracle as oracle;

/// "Distribute two +1/+1 counters among one or two target creatures." The targets and their shares are fixed as the spell
/// goes on the stack, each target getting at least one (CR 601.2d); a target illegal as it resolves takes its share with it
/// (CR 608.2b).
fn distribute(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counters_divided(g, item, CounterKind::PlusOne);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE.between(1, 2)], distribute),
    divide: Some(Divide { total: 2, reads: None }),
    ways: &[beam_me_up::way(mana("{2}{W}"), "Beam me up {2}{W}")],
    ..oracle::PURE_ENERGY
};
