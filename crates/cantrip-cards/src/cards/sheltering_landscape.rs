use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Mountain) }, { lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Plains) }>(
            "{T}, Sacrifice this land: Search your library for a basic Mountain, Forest, or Plains card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{R}{G}{W}"), "Cycling {R}{G}{W} ({R}{G}{W}, Discard this card: Draw a card.)"),
    ],
    ..oracle::SHELTERING_LANDSCAPE
};
