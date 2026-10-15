use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Once during each of your turns, you may cast a permanent spell with mana value 2 or less from your graveyard": a way
/// a permanent gives (CR 604, 118.9), for the card's own mana cost. It is used up by a cast this turn, which the history
/// notes against the permanent (`GRANTED_WAY_USED`).
fn cast_cheap_permanent(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    if player != asker.controller || q.state().turn.active != player {
        return None;
    }
    let c = q.chars(card);
    if c.is(CardType::Instant) || c.is(CardType::Sorcery) || q.mana_value(card) > 2 {
        return None;
    }
    let used = q
        .history()
        .select(EventQuery::kind(EventMask::ABILITY_ACTIVATED).object(asker.source).this_turn())
        .any(|r| matches!(r.event, Event::AbilityActivated { ability, .. } if ability.index == GRANTED_WAY_USED));
    (!used).then(|| Cost::mana(q.card(c.card).cost))
}

/// Companion is a start-of-game action the deck builder owns (CR 702.139).
pub static CARD: CardDef = CardDef {
    granted_ways: &[GrantedWay { text: "Once during each of your turns, you may cast a permanent spell with mana value 2 or less from your graveyard.", cost: cast_cheap_permanent, ..GrantedWay::CAST }],
    ..oracle::LURRUS_OF_THE_DREAM_DEN
};
