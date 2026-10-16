use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Landfall — Whenever a land you control enters, put a +1/+1 counter on target creature you control and a plan counter
/// on this enchantment." (CR 207.2c, 122.6) An ability whose target has become illegal does nothing at all, the plan
/// counter included (CR 608.2b); the counter goes only on the enchantment that triggered, if it is still there (CR 400.7).
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    g.add_counters(item.object, CounterKind::Plan, 1);
    Ok(())
}

/// "When the fourth plan counter is put on this enchantment, sacrifice it. When you do, ..." (CR 701.21a, 603.12): the
/// reflexive ability triggers only if the enchantment was sacrificed, and is put on the stack with its target only if
/// there is a creature to target.
fn sacrifice_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.sacrifice(&[item.object]).is_empty() {
        return Ok(());
    }
    let this = g.card_id(&CARD);
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Put an indestructible counter on target creature you control." (CR 122.1b)
fn indestructible_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::Indestructible, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::LANDFALL, counters, "Landfall \u{2014} Whenever a land you control enters, put a +1/+1 counter on target creature you control and a plan counter on this enchantment.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        triggered(
            when_asked(EventMask::COUNTERS, triggers::nth_counter_put_on_this::<{ CounterKind::Plan as u8 }, 4>),
            sacrifice_it,
            "When the fourth plan counter is put on this enchantment, sacrifice it. When you do, put an indestructible counter on target creature you control.",
        ),
    ],
    delayed: &[delayed_trigger(indestructible_counter, "When you do, put an indestructible counter on target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::CLAIM_THE_KINGDOM
};
