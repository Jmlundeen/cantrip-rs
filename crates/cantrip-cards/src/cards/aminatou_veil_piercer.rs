use cantrip_api::*;
use cantrip_mechanics::{effects, miracle, triggers};
use cantrip_oracle as oracle;

/// "Each enchantment card in your hand has miracle" (CR 702.94a).
fn enchantment_in_your_hand(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    miracle::in_your_hand(q, asker, card) && q.chars(card).is(CardType::Enchantment)
}

/// "Its miracle cost is equal to its mana cost reduced by {4}": its mana cost, {X} and all, reduced by {4} (`less: 4`,
/// `reduced_by`): the reduction is part of the miracle cost, so it comes before anything increases it (CR 601.2f); it takes
/// generic mana only and never below nothing, the cost's own first and then X's once X is announced (CR 107.3a, 118.7a).
/// A card with no mana cost has a miracle cost that can't be paid (CR 118.6). `card` is the face being cast, a Room's door
/// (CR 709.3a), or the whole card as it is revealed.
fn its_mana_cost(q: &dyn Query, card: CardId) -> Option<ManaCost> {
    let def = q.card(card);
    def.has_cost.then_some(def.cost)
}

pub static CARD: CardDef = CardDef {
    granted_miracles: &[GrantedMiracle {
        text: "Aminatou, Veil Piercer's miracle: its mana cost reduced by {4}",
        has: enchantment_in_your_hand,
        cost: its_mana_cost,
        less: 4,
    }],
    abilities: &[triggered(triggers::YOUR_UPKEEP, |g, item| effects::surveil(g, item, 2), "At the beginning of your upkeep, surveil 2.")],
    ..oracle::AMINATOU_VEIL_PIERCER
};
