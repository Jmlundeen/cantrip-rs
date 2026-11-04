use cantrip_api::*;
use cantrip_mechanics::triggers::creature_you_control_dies;
use cantrip_oracle as oracle;

/// "Whenever a creature you control with a +1/+1 counter on it dies" (CR 700.4, 603.10a): a creature you control dying, as
/// `creature_you_control_dies` judges it, with a +1/+1 counter on it as it last was on the battlefield. This creature too, if
/// it died with one, and each of several that die together triggers it (the ruling). Tokens count.
fn creature_with_a_counter_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, .. } if q.obj(*old).counters(CounterKind::PlusOne) == 0 => None,
        _ => creature_you_control_dies(q, me, event),
    }
}

fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, creature_with_a_counter_dies),
        draw_a_card,
        "Whenever a creature you control with a +1/+1 counter on it dies, draw a card.",
    )],
    ..oracle::MELTSTRIDER_EULOGIST
};
