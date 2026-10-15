use cantrip_api::*;
use cantrip_oracle as oracle;

/// "When The One Ring enters, if you cast it" (CR 603.4): a permanent spell that resolved is the one cast
/// (`ZoneReason::Resolve` is exactly that).
fn enters_cast(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, reason: ZoneReason::Resolve, .. } if *new == source && to.is(ZoneKind::Battlefield) => Fired::PLAIN,
        _ => None,
    }
}

/// "You gain protection from everything until your next turn" (CR 702.16j, 611.2b).
fn protect_you(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::PlayerProtectionFromEverything, Duration::UntilNextTurnOf(item.controller));
    Ok(())
}

/// "You lose 1 life for each burden counter on The One Ring", counted as the ability resolves; if it has left, as it
/// last was (CR 608.2h).
fn lose_life_for_burden(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let burdens = g.obj(item.object).counters(CounterKind::Burden);
    if burdens > 0 {
        g.lose_life(item.controller, burdens as i32);
    }
    Ok(())
}

/// "Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring."
fn add_burden_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Burden, 1);
    let burdens = g.obj(item.object).counters(CounterKind::Burden);
    if burdens > 0 {
        g.draw(item.controller, burdens as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When The One Ring enters, if you cast it, you gain protection from everything until your next turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: enters_cast,
            effect: protect_you,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your upkeep, you lose 1 life for each burden counter on The One Ring.",
            listens: EventMask::STEP_BEGAN,
            matches: cantrip_mechanics::triggers::your_upkeep,
            effect: lose_life_for_burden,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring.",
            cost: Cost { tap: true, ..Cost::NONE },
            effect: add_burden_and_draw,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::THE_ONE_RING
};
