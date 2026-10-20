use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever a Plains you control enters" (CR 305.6): a permanent with the Plains land type, by any means.
fn a_plains_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && q.chars(*new).has_subtype(Subtype::Plains) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn add_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::LANDFALL, cantrip_mechanics::effects::gain_one_life, "Landfall \u{2014} Whenever a land you control enters, you gain 1 life."),
        triggered(
            when(EventMask::ZONE_CHANGE, a_plains_you_control_enters),
            add_counter,
            "Whenever a Plains you control enters, put a +1/+1 counter on target creature.",
        )
        .targets(&[targets::CREATURE]),
    ],
    ..oracle::KOTH_OF_THE_HOMESTEAD
};
