use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "PayTwo",
    "pay {2}?",
    "If you do, put a +1/+1 counter on Proft and draw a card",
    "Pay {2}",
    "Don't pay",
);

/// "Whenever you scry or surveil, you may pay {2}. If you do, put a +1/+1 counter on Proft and draw a card" (CR 118.12):
/// the card is drawn whoever is left; the counter is put only if Proft is still the permanent it was (CR 608.2b).
fn maybe_pay(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{2}"), &ASK)? {
        if targets::permanent(g, item.object) {
            g.add_counters(item.object, CounterKind::PlusOne, 1);
        }
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_SCRY_OR_SURVEIL,
        maybe_pay,
        "Whenever you scry or surveil, you may pay {2}. If you do, put a +1/+1 counter on Proft and draw a card.",
    )],
    ..oracle::PROFT_CONSULTING_DETECTIVE
};
