use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may cast spells from your hand without paying their mana costs": an alternative cost a static ability gives (CR 118.9,
/// 601.2b) to the spells its controller casts from their hand - a card with no mana cost included (CR 118.6a) - judged on the
/// face being cast. It is the one alternative cost of that cast (CR 118.9a); additional costs, and what increases the cost,
/// are still paid (CR 118.9d, 601.2f); X is 0 (CR 107.3b). Timing is the card's own (CR 601.3), and a spell cast from anywhere
/// else is not cast from the hand.
fn from_your_hand(q: &dyn Query, asker: &Asker, spell: ObjectId, player: PlayerId) -> bool {
    player == asker.controller && q.obj(spell).zone == Zone::hand(player)
}

pub static CARD: CardDef = CardDef {
    spell_alternatives: &[SpellAlternative { text: "Without paying its mana cost", applies: from_your_hand, cost: InsteadOfManaCost::WithoutPaying }],
    ..oracle::OMNISCIENCE
};
