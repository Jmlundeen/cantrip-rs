use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Distribute two +1/+1 counters among up to two target creatures. Then if you're the monarch, double the number of
/// +1/+1 counters on each creature you control.
///
/// The targets and their shares were locked in as the ability was put on the stack (CR 601.2d, 603.3d); a target that
/// has become illegal since is taken out with its share, so those counters are lost (CR 608.2b). "If you're the
/// monarch" is asked as the ability resolves. Doubling is putting as many as a creature already has (CR 122.8), each
/// creature's count read before any is doubled.
fn distribute_then_double(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for slot in 0..item.target_count as usize {
        if let Some(creature) = targets::object(g, item, slot)
            && item.share(slot) > 0
        {
            g.add_counters(creature, CounterKind::PlusOne, item.share(slot) as i16);
        }
    }
    if g.state().player(item.controller).monarch {
        double::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, cantrip_mechanics::effects::become_monarch, "When this enchantment enters, you become the monarch."),
        triggered(cantrip_mechanics::triggers::YOUR_UPKEEP, distribute_then_double, "At the beginning of your upkeep, distribute two +1/+1 counters among up to two target creatures. Then if you're the monarch, double the number of +1/+1 counters on each creature you control.")
            .targets(&[TargetSpec { min: 0, max: 2, filter: targets::CREATURE.filter }])
            .divide(Divide { total: 2, of_x: None }),
    ],
    ..oracle::COURT_OF_GARENBRIG
};
