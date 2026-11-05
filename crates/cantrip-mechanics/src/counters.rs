//! Putting counters on a creature its controller chooses as an effect resolves.

use crate::{choose, select};
use cantrip_api::*;

/// "Put `n` `kind` counters on a creature you control" (CR 122.1, 608.2d): the creature is chosen by the ability's controller as
/// it resolves, not targeted, from the creatures they control then. A lone creature is taken without a question, and with none
/// the instruction does nothing. A creature that cannot have counters is still a legal choice; the counters just aren't put
/// (CR 609.3). Titan of Industry (a shield counter) and Humiliate (a +1/+1 counter) print it.
pub fn put_on_creature_you_control(g: &mut dyn Game, item: &StackItem, kind: CounterKind, n: i32) -> Result<(), Suspend> {
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

/// "Distribute `n` `kind` counters among any number of `among`" (CR 122.1, 608.2d): as the effect resolves, its controller decides
/// where each of the `n` counters goes, one at a time, among the creatures listed, not targeting them. A creature may get several and
/// the rest none; all `n` go on (a lone creature is taken without a question), and with none listed nothing is distributed. They are put
/// on together (CR 101.4: one event, so a "one or more counters" ability triggers once). It is not 601.2d's distribution among targets,
/// which is fixed as the spell is cast (`counters_divided`). Crashing Wave (three stun counters among tapped creatures its
/// controller's opponents control), Feast of the Victorious Dead and Blessing of Frost print it with +1/+1 counters among creatures
/// their controller controls.
pub fn distribute(g: &mut dyn Game, controller: PlayerId, source: ObjectId, among: &[ObjectId], kind: CounterKind, n: u8) -> Result<(), Suspend> {
    let mut puts: Vec<(ObjectId, CounterKind, i32)> = Vec::new();
    for _ in 0..n {
        let chosen = match among {
            [] => return Ok(()),
            [only] => Some(*only),
            _ => {
                let rule = select::Selection::of(select::exactly(1), &[select::CREATURE]).called("creature");
                choose::select(g, controller, source, Purpose::PutCounters(kind, 1), among.to_vec(), rule)?.first().copied()
            }
        };
        let Some(creature) = chosen else { continue };
        match puts.iter_mut().find(|(object, ..)| *object == creature) {
            Some((.., counters)) => *counters += 1,
            None => puts.push((creature, kind, 1)),
        }
    }
    g.add_counters_together(&puts);
    Ok(())
}
