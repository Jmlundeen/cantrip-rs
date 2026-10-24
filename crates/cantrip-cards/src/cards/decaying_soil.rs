use cantrip_api::*;
use cantrip_mechanics::{choose, triggers};
use cantrip_oracle as oracle;

const RETURN: Question = Question::new("ReturnToHand", "pay {cost} to return {subject} to your hand?", "", "Pay {cost}", "Leave it");

/// "At the beginning of your upkeep, exile a card from your graveyard." Not optional: with one card there it is that
/// card, with several its controller chooses one, and with none nothing happens.
fn exile_from_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let card = if graveyard.len() == 1 { graveyard.first().copied() } else { choose::card(g, you, item.object, Purpose::Exile, graveyard)? };
    if let Some(card) = card {
        g.exile(card);
    }
    Ok(())
}

/// Seven cards in the graveyard that were there before the event being asked about (CR 603.10a): the triggers of one
/// event see the history as it was just before it, so a card that arrived with the event is not in it. The creature whose
/// arrival makes seven does not trigger the ability, and creatures put there together all trigger it or none does.
fn threshold_before(q: &dyn Query, player: PlayerId) -> bool {
    let h = q.history();
    q.state()
        .zone(Zone::graveyard(player))
        .iter()
        .filter(|&&card| {
            h.all().iter().any(|r| matches!(r.event, Event::ZoneChange { new, to, .. } if new == card && to == Zone::graveyard(player)))
        })
        .count()
        >= 7
}

/// "Whenever a nontoken creature is put into your graveyard from the battlefield": the permanent as it left (CR 603.10a).
fn nontoken_creature_to_your_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && *to == Zone::graveyard(q.controller(source))
                && !q.obj(*old).token
                && q.chars(*old).is(CardType::Creature)
                && threshold_before(q, q.controller(source)) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "You may pay {1}. If you do, return that card to your hand." The offer is made even when the card has left the
/// graveyard since (a new object, CR 400.7), which paying then buys nothing for.
fn pay_to_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    let you = item.controller;
    if g.may_pay(you, item.object, card, cost("{1}"), &RETURN)? && is_live(g, card) && g.obj(card).zone == Zone::graveyard(you) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOUR_UPKEEP, exile_from_graveyard, "At the beginning of your upkeep, exile a card from your graveyard.")],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, nontoken_creature_to_your_graveyard), pay_to_return, "Whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {1}. If you do, return that card to your hand.")],
    statics: &[static_ability(Layer::Ability, cantrip_mechanics::statics::itself_with_threshold, cantrip_mechanics::statics::grant_delayed_ability, "Threshold \u{2014} As long as there are seven or more cards in your graveyard, this enchantment has \"Whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {1}. If you do, return that card to your hand.\"")],
    ..oracle::DECAYING_SOIL
};
