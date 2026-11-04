//! "Modified" (CR 700.9): a permanent with a counter on it, that is equipped, or that is enchanted by an Aura its controller controls.

use cantrip_api::*;

use crate::attach;
use crate::targets;

/// Whether `id` is a modified permanent (CR 700.9), through the layers: it has one or more counters of any kind on it, an Equipment
/// is attached to it (whoever controls the Equipment), or an Aura controlled by the permanent's controller is attached to it (an
/// Aura an opponent controls does not make it modified). Costume Closet and Silver Sable, Mercenary Leader ask it.
pub fn is_modified(q: &dyn Query, id: ObjectId) -> bool {
    q.obj(id).counter_array().iter().any(|&n| n > 0)
        || attach::is_equipped(q, id)
        || q.state().battlefield().iter().any(|&aura| {
            q.attached_to(aura) == Some(id) && q.chars(aura).has_subtype(Subtype::Aura) && q.controller(aura) == q.controller(id)
        })
}

fn modified_creature_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::on_battlefield(q, id)
        && q.chars(id).is(CardType::Creature)
        && q.controller(id) == asker.controller
        && is_modified(q, id))
}

/// "Target modified creature you control" (CR 115.1, 700.9). Silver Sable, Mercenary Leader prints it.
pub const MODIFIED_CREATURE_YOU_CONTROL: TargetSpec = target(modified_creature_you_control_filter);
