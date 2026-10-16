use cantrip_api::*;
use cantrip_mechanics::{choose, targets, top_of_library, triggers};
use cantrip_oracle as oracle;

/// "Whenever a Villain you control enters" (CR 603.6a): the permanent as it entered, its creature type read through this
/// enchantment's text (CR 612).
fn villain_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).has_subtype(q.words(me.source).subtype(Subtype::Villain)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Each opponent loses 1 life and you gain 1 life. Put a plan counter on this enchantment." The counter goes only on the
/// enchantment that triggered, if it is still there (CR 400.7).
fn drain_and_plan(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    g.add_counters(item.object, CounterKind::Plan, 1);
    Ok(())
}

/// "When the fifth plan counter is put on this enchantment, sacrifice it. When you do, ..." (CR 701.21a, 603.12): the
/// reflexive ability triggers only if the enchantment was sacrificed, and its target opponent is chosen as it is put on the
/// stack.
fn sacrifice_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.sacrifice(&[item.object]).is_empty() {
        return Ok(());
    }
    let this = g.card_id(&CARD);
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Target opponent exiles the top five cards of their library. You may cast up to two spells from among the exiled cards
/// without paying their mana costs." (CR 608.2g, 118.9) The cards leave together, face up; those that reached exile are
/// offered, and the rest stay in exile.
fn exile_and_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    let top = top_of_library(g, opponent, 5);
    let exiled: Vec<ObjectId> = g.exile_all(&top).into_iter().filter(|&card| g.obj(card).zone.is(ZoneKind::Exile)).collect();
    choose::cast_up_to_free(g, item.controller, item.object, &exiled, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, villain_you_control_enters),
            drain_and_plan,
            "Whenever a Villain you control enters, each opponent loses 1 life and you gain 1 life. Put a plan counter on this enchantment.",
        ),
        triggered(
            when_asked(EventMask::COUNTERS, triggers::nth_counter_put_on_this::<{ CounterKind::Plan as u8 }, 5>),
            sacrifice_it,
            "When the fifth plan counter is put on this enchantment, sacrifice it. When you do, target opponent exiles the top five cards of their library. You may cast up to two spells from among the exiled cards without paying their mana costs.",
        ),
    ],
    delayed: &[delayed_trigger(exile_and_cast, "When you do, target opponent exiles the top five cards of their library. You may cast up to two spells from among the exiled cards without paying their mana costs.")
        .targets(&[targets::OPPONENT])],
    ..oracle::DOOM_REIGNS_SUPREME
};
