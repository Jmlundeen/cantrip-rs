use cantrip_api::*;
use cantrip_mechanics::triggers::{LANDFALL, THIS_ENTERS};
use cantrip_mechanics::{earthbend, lands, targets};
use cantrip_oracle as oracle;

/// "Earthbend 2. Then search your library for a basic land card, put it onto the battlefield tapped, then shuffle." The
/// land is chosen as the ability goes on the stack, so the one searched for cannot be it (the card's ruling).
fn earthbend_then_fetch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    earthbend::effect::<2>(g, item)?;
    lands::search_basic_land_to_battlefield(g, item.controller, item.object, true)
}

/// "Whenever a land you control enters, put a quest counter on this enchantment. When you do, ..." (CR 603.12): the
/// reflexive trigger exists only if a counter was put on it, which the core does not do to an enchantment that has left
/// the battlefield.
fn quest_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let before = g.obj(item.object).counters(CounterKind::Quest);
    g.add_counters(item.object, CounterKind::Quest, 1);
    if g.obj(item.object).counters(CounterKind::Quest) > before {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 1), Target::None);
    }
    Ok(())
}

/// The reflexive trigger's intervening "if it has four or more quest counters on it" (CR 603.4), checked as it would
/// trigger and again as it resolves.
fn four_quest_counters(q: &dyn Query, asker: &Asker) -> bool {
    q.state().obj(asker.source).counters(CounterKind::Quest) >= 4
}

/// "Put a +1/+1 counter on target creature you control. It gains trample until end of turn." Nothing for a target that is
/// no longer legal (CR 608.2b).
fn counter_and_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, earthbend_then_fetch, "When this enchantment enters, earthbend 2. Then search your library for a basic land card, put it onto the battlefield tapped, then shuffle.")
            .targets(&[earthbend::TARGET]),
        triggered(LANDFALL, quest_counter, "Landfall \u{2014} Whenever a land you control enters, put a quest counter on this enchantment. When you do, if it has four or more quest counters on it, put a +1/+1 counter on target creature you control. It gains trample until end of turn."),
    ],
    delayed: &[
        earthbend::RETURN,
        delayed_trigger(counter_and_trample, "When you do, if it has four or more quest counters on it, put a +1/+1 counter on target creature you control. It gains trample until end of turn.")
            .targets(&[targets::CREATURE_YOU_CONTROL])
            .condition(four_quest_counters),
    ],
    ..oracle::EARTHBENDER_ASCENSION
};
