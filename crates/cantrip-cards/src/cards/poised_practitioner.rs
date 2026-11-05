use cantrip_api::*;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature. Scry 1." (CR 122.1, 701.22): the scry happens even if the Practitioner has left.
fn counter_then_scry(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.end_sentence();
    effects::scry(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[flurry::trigger(
        "Flurry \u{2014} Whenever you cast your second spell each turn, put a +1/+1 counter on this creature. Scry 1. (Look at the top card of your library. You may put that card on the bottom.)",
        counter_then_scry,
    )],
    ..oracle::POISED_PRACTITIONER
};
