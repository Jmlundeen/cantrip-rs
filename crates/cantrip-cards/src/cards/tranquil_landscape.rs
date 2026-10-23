use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Island) }>(
            "{T}, Sacrifice this land: Search your library for a basic Forest, Plains, or Island card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{G}{W}{U}"), "Cycling {G}{W}{U} ({G}{W}{U}, Discard this card: Draw a card.)"),
    ],
    ..oracle::TRANQUIL_LANDSCAPE
};
