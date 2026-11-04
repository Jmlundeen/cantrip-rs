use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Put a double strike counter on Mai" (CR 122.1b): a permanent with a double strike counter has double strike; on nothing
/// if she has left the battlefield (CR 400.7).
fn double_strike_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::DoubleStrike, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::PROWESS,
        ability(
            cost("{3}"),
            double_strike_counter,
            "Exhaust \u{2014} {3}: Put a double strike counter on this creature. (Activate each exhaust ability only once.)",
        )
        .exhaust(),
    ],
    ..oracle::MAI_JADED_EDGE
};
