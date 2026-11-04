//! "Modified" (CR 700.9): a permanent with a counter on it, that is equipped, or that is enchanted by an Aura its controller controls.

use cantrip_api::*;

use crate::targets;

/// Whether `id` is a modified permanent (CR 700.9), through the layers: it has one or more counters of any kind on it, an Equipment
/// is attached to it (whoever controls the Equipment), or an Aura controlled by the permanent's controller is attached to it (an
/// Aura an opponent controls does not make it modified). Costume Closet and Silver Sable, Mercenary Leader ask it.
///
/// Asked by a look-back ability ("whenever a modified creature leaves the battlefield", CR 603.10a), `q` is the game as it was just
/// before the event (the core's `BoardThen`): the permanent's counters are its last-known ones, and what was attached to it then is
/// what `permanents_of` and `attached_to` say then - an Equipment or Aura that left with it (a "destroy all nonland permanents")
/// included, one that left earlier not.
pub fn is_modified(q: &dyn Query, id: ObjectId) -> bool {
    q.obj(id).counter_array().iter().any(|&n| n > 0)
        || q.state().seats().flat_map(|p| q.permanents_of(p)).any(|attachment| {
            q.attached_to(attachment) == Some(id)
                && (q.chars(attachment).has_subtype(Subtype::Equipment)
                    || q.chars(attachment).has_subtype(Subtype::Aura) && q.controller(attachment) == q.controller(id))
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
