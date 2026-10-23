//! Enlist (CR 702.154): "As this creature attacks, you may tap up to one untapped creature you control that you didn't
//! choose to attack with and that either has haste or has been under your control continuously since this turn began. When
//! you do, this creature gets +X/+0 until end of turn, where X is the tapped creature's power." (CR 702.154a). Guardian of
//! New Benalia and Coalition Skyknight print it.
//!
//! The static ability is an optional cost to attack (CR 702.154b, 508.1g) and the triggered ability is linked to it (CR
//! 607.2h): the two are one `paid_as_it_attacks` ability, whose cost the core offers as attackers are declared and whose
//! trigger it owns (`TriggeredDef::attack_option`). A card with enlist lists `ENLIST` (or `enlist(text)`) among its
//! `abilities`, once for each instance (CR 702.154d). "Whenever this creature enlists a creature" (CR 702.154c) is
//! `THIS_ENLISTS`, about the creature enlisted.

use cantrip_api::*;

/// "When you do, this creature gets +X/+0 until end of turn, where X is the tapped creature's power": X is read as the
/// ability resolves (CR 608.2h) - the creature's power then, or as it last existed on the battlefield if it has left - and
/// a negative power gives 0 (CR 107.1b).
fn gets_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(tapped) = item.subject else { return Ok(()) };
    let x = g.chars(tapped).power.unwrap_or(0).max(0);
    crate::pump(g, item.object, item.object, x, 0);
    Ok(())
}

/// Enlist (CR 702.154a), with `text` its printed line, which a scenario's `Ability:` matches.
pub const fn enlist(text: &'static str) -> AbilityDef {
    paid_as_it_attacks(AttackOption::Enlist, gets_its_power, text)
}

/// Enlist as Guardian of New Benalia and Coalition Skyknight print it, reminder text and all.
pub const ENLIST: AbilityDef = enlist(
    "Enlist (As this creature attacks, you may tap a nonattacking creature you control without summoning sickness. When you do, add its power to this creature's until end of turn.)",
);

/// "Whenever this creature enlists a creature" (CR 702.154c): the cost of one of its enlist abilities was paid by tapping
/// that creature, which the ability is about.
pub fn this_enlists(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match *event {
        Event::PaidToAttack { attacker, option: AttackOption::Enlist, tapped, .. } if attacker == source => Fired::about(Target::Object(tapped)),
        _ => None,
    }
}

/// `this_enlists` as a trigger.
pub const THIS_ENLISTS: Trigger = when(EventMask::PAID_TO_ATTACK, this_enlists);
