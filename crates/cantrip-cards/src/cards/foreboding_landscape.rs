use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Island) }>(
            "{T}, Sacrifice this land: Search your library for a basic Swamp, Forest, or Island card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{B}{G}{U}"), "Cycling {B}{G}{U} ({B}{G}{U}, Discard this card: Draw a card.)"),
    ],
    ..oracle::FOREBODING_LANDSCAPE
};
