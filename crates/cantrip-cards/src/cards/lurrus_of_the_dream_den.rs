use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "Once during each of your turns, you may cast a permanent spell with mana value 2 or less from your graveyard": a way
/// a permanent gives (CR 604, 601.3), a permission for the card's own mana cost (`GrantedWay::permission`), so an
/// alternative cost may be paid instead. It is used up by a cast by it this turn, which the history records as made by this
/// permanent (`Cast::by`).
fn cast_cheap_permanent(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    if player != asker.controller || q.state().turn.active != player {
        return None;
    }
    let c = q.chars(card);
    if c.is(CardType::Instant) || c.is(CardType::Sorcery) || q.mana_value(card) > 2 {
        return None;
    }
    (!history::permission_used_this_turn(q, asker.source)).then(|| Cost::mana(q.card(c.card).cost))
}

/// Companion is a start-of-game action the deck builder owns (CR 702.139).
pub static CARD: CardDef = CardDef {
    granted_ways: &[GrantedWay { text: "Once during each of your turns, you may cast a permanent spell with mana value 2 or less from your graveyard.", cost: cast_cheap_permanent, permission: true, ..GrantedWay::CAST }],
    ..oracle::LURRUS_OF_THE_DREAM_DEN
};
