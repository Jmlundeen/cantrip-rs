use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Island) }, { lands::sub(Subtype::Swamp) }>(
            "{T}, Sacrifice this land: Search your library for a basic Plains, Island, or Swamp card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{W}{U}{B}"), "Cycling {W}{U}{B} ({W}{U}{B}, Discard this card: Draw a card.)"),
    ],
    ..oracle::CONTAMINATED_LANDSCAPE
};
