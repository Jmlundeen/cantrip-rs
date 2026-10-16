use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever another permanent leaves the battlefield" (CR 603.10a): any permanent but this one, a token included,
/// seen leaving the board as it was, so Shredder triggers for the others even when it leaves with them.
fn another_permanent_leaves(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. } if *old != source && from.is(ZoneKind::Battlefield) => Fired::PLAIN,
        _ => None,
    }
}

fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, another_permanent_leaves), grow, "Whenever another permanent leaves the battlefield, put a +1/+1 counter on Super Shredder.")
        .look_back()],
    ..oracle::SUPER_SHREDDER
};
