use cantrip_api::*;
use cantrip_mechanics::{effects, empower};
use cantrip_oracle as oracle;

/// "Whenever you put one or more loyalty counters on a planeswalker" (CR 122.6, 603.2): each time loyalty counters are put
/// on a planeswalker by this ability's controller, whatever their number and whoever controls the planeswalker.
fn you_put_loyalty_counters(q: &dyn Query, me: &Asker, e: &Event) -> Option<Fired> {
    match e {
        Event::CountersPut { object, kind: CounterKind::Loyalty, amount, by, .. }
            if *amount > 0 && *by == me.controller && q.chars(*object).is(CardType::Planeswalker) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::COUNTERS, you_put_loyalty_counters),
            effects::plus_one_counter_on_this,
            "Whenever you put one or more loyalty counters on a planeswalker, put a +1/+1 counter on this creature.",
        ),
        ability(cost("{6}"), |g, item| empower::jace(g, item.controller, item.object, 2), "{6}: Empower Jace 2."),
    ],
    ..oracle::INSPIRED_TETHERMAGE
};
