use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever a creature token you control enters": its own entry too, and its siblings'.
fn token_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.obj(*new).token
                && q.obj(*new).controller == q.controller(source)
                && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

fn counter_on_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(token) = item.subject
        && targets::permanent(g, token)
    {
        g.add_counters(token, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: Some(Kicker { cost: mana(3, 0, 0, 0, 0, 0), multi: true, waterbend: false }),
    abilities: &[
        triggers::SQUAD,
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever a creature token you control enters, put a +1/+1 counter on it.",
            listens: EventMask::ZONE_CHANGE,
            matches: token_enters,
            effect: counter_on_it,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SECURITRON_SQUADRON
};
