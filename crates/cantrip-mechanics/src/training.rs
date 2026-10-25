//! Training (CR 702.149a): "Whenever this creature and at least one other creature with power greater than this creature's
//! power attack, put a +1/+1 counter on this creature." Each instance triggers separately (CR 702.149b). Hopeful Initiate,
//! Apprentice Sharpshooter, Cloaked Cadet and Gryff Rider print it.
//!
//! A card with training lists `TRIGGER` among its `abilities`.

use crate::effects::plus_one_counter_on_this;
use cantrip_api::*;

/// This creature was declared an attacker together with another creature of greater power (CR 702.149a, 508.1m). The
/// powers are compared as attackers are declared, when the ability triggers: raising a power afterwards makes no training
/// trigger, and the other creature leaving or shrinking afterwards doesn't stop it (the rulings). A creature put onto the
/// battlefield attacking was never declared and triggers nothing (CR 508.4).
fn attacks_with_greater_power(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => {
            let mine = q.chars(source).power?;
            // Only another creature can have a power greater than this one's.
            let greater = q.state().attackers.iter().any(|a| q.chars(a.id).power.is_some_and(|p| p > mine));
            if greater { Fired::PLAIN } else { None }
        }
        _ => None,
    }
}

pub const TRIGGER: AbilityDef = triggered(
    when(EventMask::ATTACKS, attacks_with_greater_power),
    plus_one_counter_on_this,
    "Training (Whenever this creature attacks with another creature with greater power, put a +1/+1 counter on this creature.)",
);
