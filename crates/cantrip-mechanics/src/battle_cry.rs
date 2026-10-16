//! Battle cry (CR 702.92a): "Whenever this creature attacks, each other attacking creature gets +1/+0 until end of
//! turn." Hero of Bladehold and Sanguine Evangelist print it.
//!
//! A card with battle cry lists `TRIGGER` among its `abilities`.

use crate::pump;
use cantrip_api::*;

fn this_attacks(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => Fired::PLAIN,
        _ => None,
    }
}

fn pump_other_attackers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let others: Vec<ObjectId> = g.state().attackers.iter().map(|a| a.id).filter(|&id| id != item.object).collect();
    for id in others {
        pump(g, item.object, id, 1, 0);
    }
    Ok(())
}

pub const TRIGGER: AbilityDef = triggered(when(EventMask::ATTACKS, this_attacks), pump_other_attackers, "Battle cry (Whenever this creature attacks, each other attacking creature gets +1/+0 until end of turn.)");
