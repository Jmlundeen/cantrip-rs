use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Forest) }>(
            "{T}, Sacrifice this land: Search your library for a basic Plains, Swamp, or Forest card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{W}{B}{G}"), "Cycling {W}{B}{G} ({W}{B}{G}, Discard this card: Draw a card.)"),
    ],
    ..oracle::DECEPTIVE_LANDSCAPE
};
