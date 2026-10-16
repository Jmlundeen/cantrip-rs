use cantrip_api::*;
use cantrip_mechanics::{effects, miracle, triggers};
use cantrip_oracle as oracle;

/// "Each instant and sorcery card in your hand has miracle {2}" (CR 702.94a).
fn instant_or_sorcery_in_your_hand(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    miracle::in_your_hand(q, asker, card) && (q.chars(card).is(CardType::Instant) || q.chars(card).is(CardType::Sorcery))
}

pub static CARD: CardDef = CardDef {
    granted_miracles: &[GrantedMiracle {
        text: "Lorehold, the Historian's miracle {2}",
        has: instant_or_sorcery_in_your_hand,
        cost: |_, _| Some(mana("{2}")),
    }],
    abilities: &[triggered(
        triggers::EACH_OPPONENTS_UPKEEP,
        |g, item| effects::may_discard_to_draw(g, item, 1),
        "At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card.",
    )],
    ..oracle::LOREHOLD_THE_HISTORIAN
};
