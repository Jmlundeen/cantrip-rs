use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever another Angel you control enters" (CR 603.6a): a permanent other than this one, under your control as it
/// entered, that is an Angel (a token too). "Angel" is a word of its text (CR 612).
fn another_angel_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != me.source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).has_subtype(q.words(me.source).subtype(Subtype::Angel)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Put a +1/+1 counter on this creature."
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, another_angel_you_control_enters),
        counter,
        "Whenever another Angel you control enters, put a +1/+1 counter on this creature.",
    )],
    ..oracle::YOUTHFUL_VALKYRIE
};
