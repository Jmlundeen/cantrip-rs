use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "As an additional cost to cast this spell, discard a card" (CR 601.2b, 118.8): exactly one card, paid with the other
/// costs - the spell itself has left the hand (CR 601.2a) and can't be the card discarded. "Draw two cards" (CR 608.2c).
pub static CARD: CardDef = CardDef {
    spell: spell(&[], |g, item| effects::draw(g, item, 2)),
    // "As an additional cost to cast this spell, discard a card" (CR 601.2b, 118.8): exactly one, paid with the other costs.
    additional_cost: Some(discard_cards(1)),
    ..oracle::THRILL_OF_POSSIBILITY
};
