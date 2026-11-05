use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature or another creature you control dies, if it had counters on it" (CR 700.4, 603.4, 603.10a): main's
/// `creature_you_control_dies`, looked back on, narrowed to a creature that had counters of any kind as it last was on the
/// battlefield. Each creature is its own event, so creatures dying together trigger once each. The trigger is about that
/// creature, whose counters are put on the target.
fn creature_you_control_with_counters_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, .. } = event else { return None };
    triggers::creature_you_control_dies(q, me, event)?;
    if q.obj(*old).counter_array().iter().all(|&n| n == 0) {
        return None;
    }
    Fired::about(Target::Object(*old))
}

/// "Put its counters on up to one target creature you control": every kind the creature had, as it last was.
fn put_its_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    effects::put_counters_of_on_target(g, item, creature)
}

pub static CARD: CardDef = CardDef {
    enters: Some(effects::enters_with_plus_one_counters::<2>),
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, creature_you_control_with_counters_dies),
        put_its_counters,
        "Whenever this creature or another creature you control dies, if it had counters on it, put its counters on up to one target creature you control.",
    )
    .targets(&[up_to_one(targets::CREATURE_YOU_CONTROL.filter)])],
    ..oracle::HOST_OF_THE_HEREAFTER
};
