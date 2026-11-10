//! The tide counter cycle (CR 122.1, 603.2, 603.8): "At the beginning of your upkeep, put a tide counter on this [permanent].
//! Whenever there are four or more tide counters on this [permanent], remove all tide counters from it." Homarid and Tidal
//! Influence print it.

use cantrip_api::*;
use crate::triggers;

fn put_tide_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Tide, 1);
    Ok(())
}

/// A state trigger (CR 603.8): it triggers again only once its last instance has left the stack.
fn four_or_more(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).counters(CounterKind::Tide) >= 4
}

fn remove_all(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.remove_counters(item.object, CounterKind::Tide, i32::MAX);
    Ok(())
}

/// The two abilities, each with the card's own text.
pub const fn cycle(upkeep_text: &'static str, state_text: &'static str) -> [AbilityDef; 2] {
    [
        triggered(triggers::YOUR_UPKEEP, put_tide_counter, upkeep_text),
        state_triggered(four_or_more, remove_all, state_text),
    ]
}
