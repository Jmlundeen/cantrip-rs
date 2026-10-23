use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Mountain) }, { lands::sub(Subtype::Forest) }>(
            "{T}, Sacrifice this land: Search your library for a basic Swamp, Mountain, or Forest card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{B}{R}{G}"), "Cycling {B}{R}{G} ({B}{R}{G}, Discard this card: Draw a card.)"),
    ],
    ..oracle::TWISTED_LANDSCAPE
};
