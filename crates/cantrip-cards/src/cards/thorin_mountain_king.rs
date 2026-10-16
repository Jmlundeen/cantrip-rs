use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Any number of target Equipment you control" (CR 115.1d): an Equipment permanent its controller controls.
fn equipment_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if targets::permanent(q, id) && q.controller(id) == asker.controller && q.chars(id).has_subtype(Subtype::Equipment))
}

/// "When Thorin enters, attach any number of target Equipment you control to target creature you control. When one or more
/// Equipment become attached to that creature this way, that creature deals damage equal to its power to up to one target
/// creature."
///
/// The creature is the first target and the Equipment follow it: the core reads a variable number of targets in a slot as
/// the last slot's. A creature that is no longer a legal target (CR 608.2b) has nothing for the Equipment to be attached
/// to; each Equipment that is not a legal target stays where it is. An Equipment already attached to the creature does not
/// move and does not "become attached" (CR 701.3b), and one that can't be attached to it stays put. The second sentence is
/// a reflexive triggered ability (CR 603.12) that triggers only if at least one Equipment really became attached, carrying
/// the creature as its subject.
fn attach_equipment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let mut attached = false;
    for slot in 1..item.target_count as usize {
        let Some(equipment) = targets::object(g, item, slot) else { continue };
        let before = g.attached_to(equipment);
        g.attach(equipment, creature);
        attached |= before != Some(creature) && g.attached_to(equipment) == Some(creature);
    }
    if attached {
        let me = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(me, 0), Target::Object(creature));
    }
    Ok(())
}

/// "That creature deals damage equal to its power to up to one target creature": the damage is dealt by the creature
/// (CR 120.3), so its keywords count, and its power is read from last known information if it has left (CR 608.2h). With
/// no target chosen, or the target gone (CR 608.2b), there is no damage.
fn creature_deals_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    if g.target_legal(item, 0) {
        let power = g.chars(creature).power.unwrap_or(0).max(0) as u16;
        g.damage(creature, item.targets[0], power);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        attach_equipment,
        "When Thorin enters, attach any number of target Equipment you control to target creature you control. When one or more Equipment become attached to that creature this way, that creature deals damage equal to its power to up to one target creature.",
    )
    .targets(&[
        targets::CREATURE_YOU_CONTROL.labelled("Creature you control to attach the Equipment to"),
        up_to(TargetSpec::ANY_NUMBER, equipment_you_control).labelled("Equipment you control to attach (any number)"),
    ])],
    delayed: &[delayed_trigger(
        creature_deals_damage,
        "When one or more Equipment become attached to that creature this way, that creature deals damage equal to its power to up to one target creature.",
    )
    .targets(&[targets::CREATURE.optional().labelled("Creature to be dealt damage equal to the creature's power")])],
    ..oracle::THORIN_MOUNTAIN_KING
};
