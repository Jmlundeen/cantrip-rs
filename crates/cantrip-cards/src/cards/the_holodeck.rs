use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

const TEXT: &str = "Once during each of your turns, you may play a permanent card you own from outside the game or from your graveyard.";

/// "Once during each of your turns, you may play a permanent card you own from outside the game or from your graveyard."
/// (CR 400.11b, 601.3, 604.1, 118.9): one permission for either zone, so one use between them each turn; only during its
/// controller's own turn; only a permanent card (CR 110.4a); its cost the card's own mana cost for a spell and none for a
/// land, which the land drop pays for (CR 305.1).
fn permanent_card(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    if player != asker.controller || q.state().turn.active != player {
        return None;
    }
    if history::permission_used_this_turn(q, asker.source) {
        return None;
    }
    let c = q.chars(card);
    if c.is(CardType::Instant) || c.is(CardType::Sorcery) {
        return None;
    }
    if c.is(CardType::Land) {
        return Some(Cost::NONE);
    }
    Some(Cost::mana(q.card(c.card).cost))
}

pub static CARD: CardDef = CardDef {
    granted_ways: &[
        GrantedWay { text: TEXT, cost: permanent_card, from: ZoneKind::Graveyard, lands: true, permission: true, ..GrantedWay::CAST },
        GrantedWay { text: TEXT, cost: permanent_card, from: ZoneKind::Sideboard, lands: true, permission: true, ..GrantedWay::CAST },
    ],
    ..oracle::THE_HOLODECK
};
