use cantrip_api::*;
use cantrip_mechanics::{mana, triggers};
use cantrip_oracle as oracle;

// "This land enters tapped." (CR 614.1d) is `mana::enters_tapped`. "{T}: Add {W} or {B}" is not written here: as a Plains Swamp
// it has that mana ability from its basic land types (CR 305.6, 106.4).
pub static CARD: CardDef = CardDef {
    abilities: &[triggers::cycling(
        mana("{2}"),
        "Cycling {2} ({2}, Discard this card: Draw a card.)",
    )],
    enters: Some(mana::enters_tapped),
    ..oracle::UMBRAL_EXPANSE
};
