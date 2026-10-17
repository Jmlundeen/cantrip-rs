use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may cast spells from your hand without paying their mana costs": a way a permanent gives (CR 604, 118.9), to its
/// controller, for each nonland card in their hand - a card with no mana cost included (CR 118.6a) - as the face being
/// cast: an alternative cost of nothing. It is the one alternative cost of that cast (CR 118.9a); additional costs, and
/// what increases the cost, are still paid (CR 118.9d, 601.2f); X is 0 (CR 107.3b). Timing is the card's own (CR 601.3).
fn without_paying(_: &dyn Query, asker: &Asker, _: ObjectId, player: PlayerId) -> Option<Cost> {
    (player == asker.controller).then_some(Cost::NONE)
}

pub static CARD: CardDef = CardDef {
    granted_ways: &[GrantedWay { text: "Without paying its mana cost", cost: without_paying, from: ZoneKind::Hand, ..GrantedWay::CAST }],
    ..oracle::OMNISCIENCE
};
