//! Putting counters on a creature its controller chooses as an effect resolves.

use crate::{choose, select};
use cantrip_api::*;

/// "Put `n` `kind` counters on a creature you control" (CR 122.1, 608.2d): the creature is chosen by the ability's controller as
/// it resolves, not targeted, from the creatures they control then. A lone creature is taken without a question, and with none
/// the instruction does nothing. A creature that cannot have counters is still a legal choice; the counters just aren't put
/// (CR 609.3). Titan of Industry (a shield counter) and Humiliate (a +1/+1 counter) print it.
pub fn put_on_creature_you_control(g: &mut dyn Game, item: &StackItem, kind: CounterKind, n: i16) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let chosen = match creatures.as_slice() {
        [] => None,
        [only] => Some(*only),
        _ => {
            let purpose = Purpose::PutCounters(kind, u8::try_from(n).unwrap_or(u8::MAX));
            let rule = select::creature_permanents(select::exactly(1));
            choose::select(g, item.controller, item.object, purpose, creatures, rule)?.first().copied()
        }
    };
    if let Some(creature) = chosen {
        g.add_counters(creature, kind, n);
    }
    Ok(())
}
