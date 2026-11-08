use cantrip_api::*;
use cantrip_mechanics::{mana, triggers};
use cantrip_oracle as oracle;

/// "({T}: Add {R} or {W}.) This land enters tapped. Cycling {2} ({2}, Discard this card: Draw a card.)" (CR 305.6, 614.1d,
/// 702.29a): Mountain and Plains give the two intrinsic mana abilities (CR 305.6), the entry is the shared tapped entry, and
/// cycling is the shared cycling trigger, so the card adds no lines of its own.
pub static CARD: CardDef = CardDef {
    abilities: &[triggers::cycling(
        mana("{2}"),
        "Cycling {2} ({2}, Discard this card: Draw a card.)",
    )],
    enters: Some(mana::enters_tapped),
    ..oracle::GLITTERING_MASSIF
};
