use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature" (CR 122.1): on the Chocobo if it is still on the battlefield as the ability resolves,
/// and nowhere if it has left (counters can't be put on a card in another zone).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, triggers::landfall), counter, "Landfall \u{2014} Whenever a land you control enters, put a +1/+1 counter on this creature.")],
    ..oracle::SAZHS_CHOCOBO
};
