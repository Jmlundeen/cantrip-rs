use cantrip_api::*;
use cantrip_mechanics::{choose, triggers};
use cantrip_oracle as oracle;

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
    if g.unless_pays(you, item.object, card, mana(1, 0, 0, 0, 0, 0))? && is_live(g, card) && g.obj(card).zone == Zone::graveyard(you) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

/// "Threshold \u{2014} As long as there are seven or more cards in your graveyard, this enchantment has ...": the
/// enchantment's own delayed ability 0.
fn has_threshold(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = view.state();
    source == affected && state.zone(Zone::graveyard(state.obj(source).controller)).len() >= 7
}

fn grant_return(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of your upkeep, exile a card from your graveyard.",
        listens: EventMask::STEP_BEGAN,
        matches: triggers::your_upkeep,
        effect: exile_from_graveyard,
        ..TriggeredDef::BLANK
    })],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {1}. If you do, return that card to your hand.",
        listens: EventMask::ZONE_CHANGE,
        matches: nontoken_creature_to_your_graveyard,
        effect: pay_to_return,
        look_back: true,
        ..TriggeredDef::BLANK
    })],
    statics: &[StaticDef {
        text: "Threshold \u{2014} As long as there are seven or more cards in your graveyard, this enchantment has \"Whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {1}. If you do, return that card to your hand.\"",
        layer: Layer::Ability,
        affects: has_threshold,
        apply: grant_return,
        ..StaticDef::BLANK
    }],
    ..oracle::DECAYING_SOIL
};
