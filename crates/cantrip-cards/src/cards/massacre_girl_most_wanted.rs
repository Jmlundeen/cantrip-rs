use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever another creature or planeswalker you control dies" (CR 700.4, 603.10a): judged as it last was on the
/// battlefield, under the controller it last had, and never Massacre Girl herself.
fn another_creature_or_planeswalker_you_control_dies(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old != source
                && from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.controller(*old) == q.controller(source)
                && {
                    let c = q.chars(*old);
                    c.is(CardType::Creature) || c.is(CardType::Planeswalker)
                } =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Massacre Girl deals 1 damage to target opponent and you gain 1 life." One target; if it is illegal the whole
/// ability does nothing (CR 608.2b).
fn damage_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = targets::player(g, item, 0) {
        g.damage(item.object, Target::Player(opponent), 1);
    }
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "Whenever an opponent is dealt noncombat damage" (CR 120.3): any source's, to any opponent of the source's controller.
fn an_opponent_is_dealt_noncombat_damage(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { target: Target::Player(player), amount, combat: false, .. }
            if *amount > 0 && q.state().opponents(q.controller(source)).any(|o| o == *player) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn add_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask::ZONE_CHANGE, another_creature_or_planeswalker_you_control_dies),
            damage_and_gain,
            "Whenever another creature or planeswalker you control dies, Massacre Girl deals 1 damage to target opponent and you gain 1 life.",
        )
        .targets(&[targets::OPPONENT]),
        triggered(
            when(EventMask::DAMAGE, an_opponent_is_dealt_noncombat_damage),
            add_counter,
            "Whenever an opponent is dealt noncombat damage, put a +1/+1 counter on Massacre Girl.",
        ),
    ],
    ..oracle::MASSACRE_GIRL_MOST_WANTED
};
