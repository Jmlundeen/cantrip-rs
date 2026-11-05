use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// Melds with Vanille, Cheerful l'Cie into Ragnarok, Divine Deliverance (CR 712.4a): the content's `meld`, which Vanille's
/// ability carries out.
pub static CARD: CardDef = CardDef {
    abilities: &[
        // "Whenever one or more cards leave your graveyard" (CR 603.2c): the cards that left together trigger it once.
        triggered(triggers::CARDS_LEAVE_YOUR_GRAVEYARD, effects::draw_a_card_and_lose_1_life, "Whenever one or more cards leave your graveyard, you draw a card and you lose 1 life. This ability triggers only once each turn.")
            .once_per_batch()
            .triggers_each_turn(1),
    ],
    meld: Some(Meld { with: "Vanille, Cheerful l'Cie", into: &super::vanille_cheerful_lcie::RAGNAROK }),
    ..oracle::FANG_FEARLESS_LCIE
};
