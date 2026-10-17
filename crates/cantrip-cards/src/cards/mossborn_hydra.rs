use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers};
use cantrip_oracle as oracle;

/// "This creature enters with a +1/+1 counter on it" (CR 614.1c, 122.6): the counter is on it as it enters, so it is never a
/// 0/0 on the battlefield.
fn enters_with_a_counter(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += 1;
    Ok(())
}

/// "Double the number of +1/+1 counters on this creature" (CR 701.10e): put as many more as it has as the ability resolves. It
/// does nothing if the Hydra has left the battlefield (counters can't be put on it there).
fn double_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        double::counters(g, &[item.object], CounterKind::PlusOne);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_a_counter),
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, triggers::landfall), double_counters, "Landfall \u{2014} Whenever a land you control enters, double the number of +1/+1 counters on this creature.")],
    ..oracle::MOSSBORN_HYDRA
};
