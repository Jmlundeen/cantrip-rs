use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        triggers::cycling(
            mana("{2}"),
            "Cycling {2} ({2}, Discard this card: Draw a card.)",
        ),
    ],
    ..oracle::BLASTED_LANDSCAPE
};
