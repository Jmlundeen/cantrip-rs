use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Choose a card name. Search target opponent's graveyard, hand, and library for up to four cards with that name and exile them.
/// That player shuffles, then draws a card for each card exiled from their hand this way." (CR 701.23, 406.2, 608.2c).
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], library::exile_up_to_four_named_and_draw_for_each_from_hand),
    ..oracle::UNMOORED_EGO
};
