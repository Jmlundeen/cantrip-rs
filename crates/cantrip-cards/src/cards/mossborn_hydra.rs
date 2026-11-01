use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers};
use cantrip_oracle as oracle;

/// "Double the number of +1/+1 counters on this creature" (CR 701.10e): put as many more as it has as the ability resolves. It
/// does nothing if the Hydra has left the battlefield (counters can't be put on it there).
fn double_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        double::counters(g, &[item.object], CounterKind::PlusOne);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::effects::enters_with_a_counter::<{ CounterKind::PlusOne as usize }>),
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, triggers::landfall), double_counters, "Landfall \u{2014} Whenever a land you control enters, double the number of +1/+1 counters on this creature.")],
    ..oracle::MOSSBORN_HYDRA
};
