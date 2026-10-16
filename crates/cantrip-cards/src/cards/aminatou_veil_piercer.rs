use cantrip_api::*;
use cantrip_mechanics::{effects, miracle, triggers};
use cantrip_oracle as oracle;

/// "Each enchantment card in your hand has miracle" (CR 702.94a).
fn enchantment_in_your_hand(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    miracle::in_your_hand(q, asker, card) && q.chars(card).is(CardType::Enchantment)
}

/// "Its miracle cost is equal to its mana cost reduced by {4}": the generic part of its mana cost, less four, and the rest
/// as printed (CR 601.2f: a reduction by generic mana takes off generic mana only, and never below nothing); an {X} in it
/// stays, its value chosen as the card is cast (CR 107.3a). A card with no mana cost has a miracle cost that can't be paid
/// (CR 118.6). `card` is the face being cast, a Room's door (CR 709.3a), or the whole card as it is revealed.
fn mana_cost_less_four(q: &dyn Query, card: CardId) -> Option<ManaCost> {
    let def = q.card(card);
    def.has_cost.then(|| ManaCost { generic: def.cost.generic.saturating_sub(4), ..def.cost })
}

pub static CARD: CardDef = CardDef {
    granted_miracles: &[GrantedMiracle {
        text: "Aminatou, Veil Piercer's miracle: its mana cost reduced by {4}",
        has: enchantment_in_your_hand,
        cost: mana_cost_less_four,
    }],
    abilities: &[triggered(triggers::YOUR_UPKEEP, |g, item| effects::surveil(g, item, 2), "At the beginning of your upkeep, surveil 2.")],
    ..oracle::AMINATOU_VEIL_PIERCER
};
