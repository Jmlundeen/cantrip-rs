use cantrip_api::*;
use cantrip_mechanics::tokens::{self, investigate};
use cantrip_oracle as oracle;

fn clue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    investigate(g, item.controller)
}

/// "Whenever you sacrifice a Clue" (CR 701.21): a Clue of yours moved from the battlefield by a sacrifice, read as it
/// was when it left.
fn you_sacrifice_a_clue(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, reason: ZoneReason::Sacrifice, .. }
            if from.is(ZoneKind::Battlefield)
                && q.controller(*old) == q.controller(source)
                && q.chars(*old).subtypes.has(Subtype::Clue) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::LANDFALL, clue, "Landfall \u{2014} Whenever a land you control enters, investigate."),
        triggered(when(EventMask::ZONE_CHANGE, you_sacrifice_a_clue), grow, "Whenever you sacrifice a Clue, put a +1/+1 counter on Tireless Tracker.")
            .on_sacrifice()
            .look_back(),
    ],
    ..oracle::TIRELESS_TRACKER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
