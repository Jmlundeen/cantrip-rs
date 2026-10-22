use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Whenever one or more creatures deal combat damage to you while you're the monarch" (CR 603.2c, 725.1): one event for each
/// creature that dealt some of it to the controller, its subject that creature, so the ability - being `once_per_batch` -
/// triggers once for the batch, its members those creatures. Damage of 0 is not dealt, so a 0-power attacker is nothing.
fn creature_deals_combat_damage_to_you(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source, target: Target::Player(p), amount, combat: true, .. }
            if *amount > 0 && *p == me.controller && q.state().player(me.controller).monarch =>
        {
            Fired::about(Target::Object(*source))
        }
        _ => None,
    }
}

/// "Tap those creatures and put a stun counter on each of them" (CR 701.26a, 122.1d): each of them that is still a
/// permanent, tapped and given its counter. One that died from that same combat damage is no permanent to tap.
fn tap_them_and_stun(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for id in g.event_objects(item).to_vec() {
        if g.is_on_battlefield(id) {
            g.tap(id);
            g.add_counters(id, CounterKind::Stun, 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::become_monarch, "When Tamiyo enters, you become the monarch."),
        triggered(
            when_asked(EventMask::DAMAGE, creature_deals_combat_damage_to_you),
            tap_them_and_stun,
            "Whenever one or more creatures deal combat damage to you while you're the monarch, tap those creatures and put a stun counter on each of them.",
        )
        .once_per_batch(),
    ],
    ..oracle::TAMIYO_UPRISER_CROWNED
};
