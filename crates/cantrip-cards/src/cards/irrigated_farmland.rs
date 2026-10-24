use cantrip_api::*;
use cantrip_mechanics::{mana, triggers};
use cantrip_oracle as oracle;

/// "({T}: Add {W} or {U}.)" comes from its basic land types (CR 305.6); "This land enters tapped." and
/// "Cycling {2}" (CR 702.29) are the shared mechanics.
pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[triggers::cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::IRRIGATED_FARMLAND
};
