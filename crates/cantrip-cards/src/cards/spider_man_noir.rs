use cantrip_api::*;
use cantrip_mechanics::effects::{counters_on, surveil};
use cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_ATTACKS_ALONE;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on it. Then surveil X, where X is the number of counters on it." (CR 122.1, 701.25): "it" is the creature that
/// attacked alone; X counts counters of every kind, read after the counter is put on it (or as it last existed, if it has left
/// the battlefield, CR 608.2h).
fn counter_then_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.end_sentence();
    let x = counters_on(g, creature) as usize;
    surveil(g, item, x)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        CREATURE_YOU_CONTROL_ATTACKS_ALONE,
        counter_then_surveil,
        "Whenever a creature you control attacks alone, put a +1/+1 counter on it. Then surveil X, where X is the number of counters on it.",
    )],
    ..oracle::SPIDER_MAN_NOIR
};
