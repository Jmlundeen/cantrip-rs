use cantrip_api::*;
use cantrip_mechanics::replacements::WIN_INSTEAD_OF_DRAWING_FROM_AN_EMPTY_LIBRARY;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "If you would draw a card while your library has no cards in it, you win the game instead." (CR 614.1a, 121.6.)
    replacements: &[WIN_INSTEAD_OF_DRAWING_FROM_AN_EMPTY_LIBRARY],
    ..oracle::LABORATORY_MANIAC
};
