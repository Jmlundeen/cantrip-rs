use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Island) }, { lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Mountain) }>(
            "{T}, Sacrifice this land: Search your library for a basic Island, Swamp, or Mountain card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{U}{B}{R}"), "Cycling {U}{B}{R} ({U}{B}{R}, Discard this card: Draw a card.)"),
    ],
    ..oracle::SEETHING_LANDSCAPE
};
