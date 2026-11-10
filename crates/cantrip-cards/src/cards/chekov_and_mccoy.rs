use cantrip_api::*;
use cantrip_mechanics::effects::may_draw_that_many_once_each_turn;
use cantrip_oracle as oracle;

/// "Whenever you put one or more +1/+1 counters on Chekov and McCoy" (CR 122.6, 603.2c): the player who put them is the
/// event's, so a counter an opponent's spell or ability puts on him does not trigger the ability, while one he puts there
/// himself does; it triggers once however many are put at once, and the number put carries as the number to draw. Not at all
/// once its controller has drawn with it this turn (CR 603.2h).
fn you_put_plus_one_counters_on_this(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::PlusOne, amount, by, .. }
            if *object == source
                && *amount > 0
                && *by == q.controller(source)
                && !q.history().done(source, AbilityRef { card: q.chars(source).card, index: 0 }, q.controller(source)) =>
        {
            Some(Fired { subject: Target::Object(source), data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::COUNTERS, you_put_plus_one_counters_on_this),
        may_draw_that_many_once_each_turn,
        "Whenever you put one or more +1/+1 counters on Chekov and McCoy, you may draw that many cards. Do this only once each turn.",
    )],
    ..oracle::CHEKOV_AND_MCCOY
};
