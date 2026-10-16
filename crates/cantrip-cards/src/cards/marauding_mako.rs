use cantrip_api::*;
use cantrip_mechanics::triggers::cycling;
use cantrip_oracle as oracle;

/// "Whenever you discard one or more cards" (CR 603.2c): asked of each card of the discard, and it triggers once for
/// the cards discarded together. "You discard" is a card leaving its owner's hand (CR 701.9a), so the owner is asked,
/// not the destination: a card madness exiles was still discarded (CR 702.35a).
fn you_discard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { player, card } if *player == q.controller(source) => Fired::about(Target::Object(*card)),
        _ => None,
    }
}

/// "Put that many +1/+1 counters": the size of the batch the trigger saw, counted even for a card that has since left.
fn that_many_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let many = g.event_objects(item).len() as i16;
    g.add_counters(item.object, CounterKind::PlusOne, many);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::DISCARDED, you_discard), that_many_counters, "Whenever you discard one or more cards, put that many +1/+1 counters on this creature.")
            .once_per_batch(),
        cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)"),
    ],
    ..oracle::MARAUDING_MAKO
};
