use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Island) }, { lands::sub(Subtype::Mountain) }, { lands::sub(Subtype::Plains) }>(
            "{T}, Sacrifice this land: Search your library for a basic Island, Mountain, or Plains card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{U}{R}{W}"), "Cycling {U}{R}{W} ({U}{R}{W}, Discard this card: Draw a card.)"),
    ],
    ..oracle::PERILOUS_LANDSCAPE
};
