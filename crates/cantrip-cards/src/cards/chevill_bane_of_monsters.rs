use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "A permanent with a bounty counter on it", as it is on the battlefield now.
fn has_bounty(q: &dyn Query, id: ObjectId) -> bool {
    q.obj(id).counters(CounterKind::Bounty) > 0
}

/// "If your opponents control no permanents with bounty counters on them" (CR 603.4): an intervening if, asked as the
/// upkeep begins and again as the ability resolves. Any bounty counter counts, whoever put it there (the ruling).
fn opponents_control_no_bounty(q: &dyn Query, asker: &Asker) -> bool {
    !q.state().battlefield().iter().any(|&id| q.controller(id) != asker.controller && has_bounty(q, id))
}

/// "Put a bounty counter on target creature or planeswalker an opponent controls" (CR 122.1). With the target illegal
/// the ability does not resolve (CR 608.2b).
fn put_bounty_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(target, CounterKind::Bounty, 1);
    Ok(())
}

/// "Whenever a permanent an opponent controls with a bounty counter on it dies" (CR 700.4, 603.10a): looked back on, so
/// the permanent is judged as it last was on the battlefield - its controller, its bounty counters - and it need no
/// longer have been a creature or a planeswalker (the ruling).
fn opposing_bounty_permanent_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && has_bounty(q, *old)
                && q.controller(*old) != me.controller =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            YOUR_UPKEEP,
            put_bounty_counter,
            "At the beginning of your upkeep, if your opponents control no permanents with bounty counters on them, put a bounty counter on target creature or planeswalker an opponent controls.",
        )
        .targets(&[targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS])
        .condition(opponents_control_no_bounty),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, opposing_bounty_permanent_dies),
            effects::gain_life_then_draw::<3, 1>,
            "Whenever a permanent an opponent controls with a bounty counter on it dies, you gain 3 life and draw a card.",
        ),
    ],
    ..oracle::CHEVILL_BANE_OF_MONSTERS
};
