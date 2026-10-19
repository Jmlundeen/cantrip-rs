use cantrip_api::*;
use cantrip_mechanics::effects::{draw, plus_one_counter_on_this};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever you put one or more +1/+1 counters on this creature" (CR 122.6, 603.2c): the player who put them is the
/// event's, so counters an opponent puts on it do not trigger the ability; it triggers once however many are put at once.
fn you_put_plus_one_counters_on_this(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::PlusOne, amount, by, .. }
            if *object == source && *amount > 0 && *by == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_GAIN_LIFE, plus_one_counter_on_this, "Whenever you gain life, put a +1/+1 counter on this creature."),
        triggered(when(EventMask::COUNTERS, you_put_plus_one_counters_on_this), |g, item| draw(g, item, 1), "Whenever you put one or more +1/+1 counters on this creature, draw a card. This ability triggers only once each turn.")
            .triggers_each_turn(1),
    ],
    ..oracle::EXEMPLAR_OF_LIGHT
};
