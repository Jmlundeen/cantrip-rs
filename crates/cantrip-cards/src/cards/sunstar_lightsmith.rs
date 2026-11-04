use cantrip_api::*;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature and draw a card." (CR 122.1, 121.1): both, in one sentence; the counter only if it is
/// still the object that triggered (CR 400.7), the card whatever became of it.
fn counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[flurry::trigger(
        "Whenever you cast your second spell each turn, put a +1/+1 counter on this creature and draw a card.",
        counter_and_draw,
    )],
    ..oracle::SUNSTAR_LIGHTSMITH
};
