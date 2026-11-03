use cantrip_api::*;
use cantrip_mechanics::{choose_from_hand, effects, hand, targets};
use cantrip_oracle as oracle;

/// "Whenever one or more creatures you control with flying attack" (CR 603.2c): asked of each creature declared as an attacker,
/// those that its controller controls and that have flying as the attack is declared, and one trigger for the batch.
fn fliers_you_control_attack(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.controller(*attacker) == q.controller(source) && q.chars(*attacker).keywords.has(Keyword::Flying) => {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

/// "Draw a card, then discard a card. When you discard a nonland card this way, put a +1/+1 counter on target creature you
/// control." The card is drawn first, so it may be the one discarded (CR 701.9); the discard is not optional, and an empty hand
/// has none. The reflexive ability (CR 603.12) triggers only when a nonland card was discarded - one put into a hidden zone
/// unrevealed is undefined, no nonland card (CR 701.9c, `hand::discarded_nonland_card`) - and its target is chosen as it goes
/// on the stack.
fn loot_then_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 1);
    let chosen = choose_from_hand(g, you, item.object, 1)?;
    let discarded = g.discard_together(&chosen);
    if discarded.iter().any(|&card| hand::discarded_nonland_card(&*g, card)) {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, you, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ATTACKS, fliers_you_control_attack),
        loot_then_counter,
        "Whenever one or more creatures you control with flying attack, draw a card, then discard a card. When you discard a nonland card this way, put a +1/+1 counter on target creature you control.",
    )
    .once_per_batch()],
    delayed: &[delayed_trigger(effects::plus_one_counter_on_target, "When you discard a nonland card this way, put a +1/+1 counter on target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::TEO_SPIRITED_GLIDER
};
