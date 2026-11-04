//! "Modified" (CR 700.9): a permanent with a counter on it, that is equipped, or that is enchanted by an Aura its controller controls.

use cantrip_api::*;

use crate::targets;

/// Whether `id` is a modified permanent (CR 700.9), through the layers: it has one or more counters of any kind on it, an Equipment
/// is attached to it (whoever controls the Equipment), or an Aura controlled by the permanent's controller is attached to it (an
/// Aura an opponent controls does not make it modified). Costume Closet and Silver Sable, Mercenary Leader ask it.
///
/// Asked of a permanent that has just left the battlefield, it answers for the permanent as it last was (CR 603.10a, "whenever a modified
/// creature leaves the battlefield"): its counters are its last-known ones, and an Equipment or Aura still on the battlefield names it in its
/// own `attached_to` until state-based actions unattach it (CR 704.5m, 704.5n), which `Query::attached_to` would already deny. One that left
/// in the same event (a "destroy all nonland permanents") is judged as it last existed too: its old row still names the permanent it was on,
/// and its leaving is not yet in the history, which ends before the events whose triggers are being looked for (CR 603.10a).
pub fn is_modified(q: &dyn Query, id: ObjectId) -> bool {
    q.obj(id).counter_array().iter().any(|&n| n > 0)
        || q.state().battlefield().iter().any(|&attachment| makes_it_modified(q, attachment, id))
        || q.obj(id).became.is_some() && (0..q.state().objects.len() as u32).map(ObjectId).any(|attachment| left_with_it(q, attachment, id))
}

/// Whether `attachment`, as it is or last was, is an Equipment attached to `id` or an Aura attached to it under its controller (CR 700.9).
fn makes_it_modified(q: &dyn Query, attachment: ObjectId, id: ObjectId) -> bool {
    q.obj(attachment).attached_to == Some(id)
        && (q.chars(attachment).has_subtype(Subtype::Equipment)
            || q.chars(attachment).has_subtype(Subtype::Aura) && q.controller(attachment) == q.controller(id))
}

/// Whether `attachment` left the battlefield attached to `id` in the event being judged: it has left (CR 400.7), from the battlefield,
/// and no record of the history before that event is of its leaving.
fn left_with_it(q: &dyn Query, attachment: ObjectId, id: ObjectId) -> bool {
    let row = q.obj(attachment);
    row.attached_to == Some(id)
        && row.became.is_some()
        && row.zone.is(ZoneKind::Battlefield)
        && !q.history().all().iter().any(|r| matches!(r.event, Event::ZoneChange { old, .. } if old == attachment))
        && makes_it_modified(q, attachment, id)
}

fn modified_creature_you_control_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::on_battlefield(q, id)
        && q.chars(id).is(CardType::Creature)
        && q.controller(id) == asker.controller
        && is_modified(q, id))
}

/// "Target modified creature you control" (CR 115.1, 700.9). Silver Sable, Mercenary Leader prints it.
pub const MODIFIED_CREATURE_YOU_CONTROL: TargetSpec = target(modified_creature_you_control_filter);
