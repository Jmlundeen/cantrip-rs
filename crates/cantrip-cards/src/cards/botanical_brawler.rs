use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "This creature enters with two +1/+1 counters on it. Whenever one or more +1/+1 counters are put on another permanent you control, if it's the first time +1/+1 counters have been put on that permanent this turn, put a +1/+1 counter on this creature." (CR 614.1c, 122.6).
fn creature_enters_two_1_1_enters(
    _: &mut dyn Game,
    entering: &mut Entering,
) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += 2;
    Ok(())
}

/// "Whenever one or more +1/+1 counters are put on another permanent you control" (CR 603.2).
fn when_one_more_1_1_counters(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::CountersPut { object, kind, amount, .. } = event else { return None };
    let id = *object;
    let first_time_this_turn = !q
        .history()
        .this_turn()
        .iter()
        .any(|r| matches!(r.event, Event::CountersPut { object, kind: CounterKind::PlusOne, .. } if object == id));
    if *kind == CounterKind::PlusOne && asker.source != id && asker.controller == q.controller(id) && first_time_this_turn {
        return Some(Fired { subject: Target::Object(id), data: [*amount as u32, id.0] });
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
                when_asked(EventMask::COUNTERS, when_one_more_1_1_counters),
                effects::plus_one_counter_on_this,
                "Whenever one or more +1/+1 counters are put on another permanent you control, if it's the first time +1/+1 counters have been put on that permanent this turn, put a +1/+1 counter on this creature.",
            )
            .once_per_batch()
            .batch_by_data(),
    ],
    enters: Some(creature_enters_two_1_1_enters),
    ..oracle::BOTANICAL_BRAWLER
};
