use cantrip_api::*;
use cantrip_mechanics::{effects, select, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, you may sacrifice an artifact or creature. If you do, draw a card." The sacrifice is chosen as
/// the ability resolves and may be declined (CR 608.2d, 701.21a); this creature is itself a creature to sacrifice, and the card
/// is drawn only if a permanent was actually sacrificed (CR 608.2c).
fn sacrifice_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::artifact_or_creature_permanents(select::up_to(1));
    if !effects::may_sacrifice(g, item.controller, item.object, rule)?.is_empty() {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

/// "Whenever another creature you control dies, if it had counters on it" (CR 603.4, 603.10a): the creature as it last was on
/// the battlefield, under the controller it last had, with counters of any kind. Each creature is its own event, so creatures
/// dying together trigger once each, and this permanent leaving in the same event sees them. The trigger is about that creature,
/// whose counters are put on this one.
fn another_creature_you_control_with_counters_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::another_creature_you_control_dies(q, me, event)?;
    match event {
        Event::ZoneChange { old, .. } if q.obj(*old).counter_array().iter().any(|&n| n != 0) => Fired::about(Target::Object(*old)),
        _ => None,
    }
}

/// "Put its counters on this creature": every kind the creature had as it last was, together (CR 122.6); this creature is not a
/// new object once it has left the battlefield (CR 400.7).
fn put_its_counters_on_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(dead) = item.subject else { return Ok(()) };
    effects::put_counters_of_on(g, dead, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            sacrifice_to_draw,
            "When this creature enters, you may sacrifice an artifact or creature. If you do, draw a card.",
        ),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_creature_you_control_with_counters_dies),
            put_its_counters_on_this,
            "Whenever another creature you control dies, if it had counters on it, put its counters on this creature.",
        ),
    ],
    ..oracle::BUZZARD_WASP_COLONY
};
