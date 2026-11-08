use cantrip_api::*;
use cantrip_mechanics::{mana, triggers};
use cantrip_oracle as oracle;

/// "This land enters tapped." (CR 614.1d), as `mana::enters_tapped`. Its Forest and Island subtypes give "{T}: Add {G} or {U}" on their own (CR
/// 305.6), so no mana ability is printed here. "Cycling {2}" (CR 702.29a) is the discard ability, from the hand only.
pub static CARD: CardDef = CardDef {
    abilities: &[triggers::cycling(
        mana("{2}"),
        "Cycling {2} ({2}, Discard this card: Draw a card.)",
    )],
    enters: Some(mana::enters_tapped),
    ..oracle::RAIN_SLICKED_COPSE
};
