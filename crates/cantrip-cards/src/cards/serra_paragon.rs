use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "Once during each of your turns, you may play a land from your graveyard or cast a permanent spell with mana value 3 or
/// less from your graveyard." A way a permanent gives (CR 604, 118.9): the cost is the card's own mana cost for a spell and
/// none for a land (the land drop is the engine's): a permission (`GrantedWay::permission`), so an alternative cost may be
/// paid instead. One use of either, by this permanent, spends it: the history records each as made by it.
fn from_graveyard(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    if player != asker.controller || q.state().turn.active != player {
        return None;
    }
    if history::permission_used_this_turn(q, asker.source) {
        return None;
    }
    let c = q.chars(card);
    if c.is(CardType::Land) {
        return Some(Cost::NONE);
    }
    if c.is(CardType::Instant) || c.is(CardType::Sorcery) || q.mana_value(card) > 3 {
        return None;
    }
    Some(Cost::mana(q.card(c.card).cost))
}

/// "When this permanent is put into a graveyard from the battlefield" (CR 700.4), judged as it was.
fn put_into_graveyard(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. } if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Exile it and you gain 2 life": the card is exiled if it is still in the graveyard it went to; the life is gained either way.
fn exile_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = g.found_after_move(item.object) {
        g.exile(card);
    }
    g.gain_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    granted_ways: &[GrantedWay {
        text: "Once during each of your turns, you may play a land from your graveyard or cast a permanent spell with mana value 3 or less from your graveyard.",
        cost: from_graveyard,
        lands: true,
        gains: Some(0),
        permission: true,
        ..GrantedWay::CAST
    }],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, put_into_graveyard), exile_and_gain, "When this permanent is put into a graveyard from the battlefield, exile it and you gain 2 life.")
        .look_back()],
    ..oracle::SERRA_PARAGON
};
