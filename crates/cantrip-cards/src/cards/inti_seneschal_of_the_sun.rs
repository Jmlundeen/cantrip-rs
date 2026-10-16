use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "You may discard a card. When you do, ...": the discard is a choice made as the ability resolves (CR 701.9), not a
/// cost, so a player with no cards is not asked. The reflexive trigger (CR 603.12) follows a card really discarded.
fn discard_for_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    let chosen = choose::cards(g, item.controller, item.object, Purpose::Discard, hand, 1, true)?;
    let Some(&card) = chosen.first() else { return Ok(()) };
    g.discard(card);
    let this = g.card_id(&CARD);
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Target attacking creature" (CR 115.1, 506.4): declared as an attacker and still attacking.
fn attacking_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.state().attackers.iter().any(|a| a.id == id))
}

/// "Put a +1/+1 counter on target attacking creature. It gains trample until end of turn."
fn empower(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    Ok(())
}

/// "Whenever you discard one or more cards" (CR 603.2c): once for the cards discarded together.
fn you_discard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { player, card } if *player == q.controller(source) => Fired::about(Target::Object(*card)),
        _ => None,
    }
}

/// "Exile the top card of your library. You may play that card until your next end step": a permission to play, not
/// only to cast, so a land may be played as the turn's land drop (CR 305.1). Taking it is not a draw.
fn exile_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let grant = PlayGrant { lands: true, expiry: Expiry::UntilYourNextEndStep, ..PlayGrant::CAST };
    cantrip_mechanics::library::exile_top_may_play(g, item.controller, 1, item.controller, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::YOU_ATTACK, discard_for_counter, "Whenever you attack, you may discard a card. When you do, put a +1/+1 counter on target attacking creature. It gains trample until end of turn."),
        triggered(when(EventMask::DISCARDED, you_discard), exile_and_play, "Whenever you discard one or more cards, exile the top card of your library. You may play that card until your next end step.")
            .once_per_batch(),
    ],
    delayed: &[delayed_trigger(empower, "When you do, put a +1/+1 counter on target attacking creature. It gains trample until end of turn.")
        .targets(&[target(attacking_creature)])],
    ..oracle::INTI_SENESCHAL_OF_THE_SUN
};
