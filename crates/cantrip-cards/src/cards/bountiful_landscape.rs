use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Island) }, { lands::sub(Subtype::Mountain) }>(
            "{T}, Sacrifice this land: Search your library for a basic Forest, Island, or Mountain card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{G}{U}{R}"), "Cycling {G}{U}{R} ({G}{U}{R}, Discard this card: Draw a card.)"),
    ],
    ..oracle::BOUNTIFUL_LANDSCAPE
};
