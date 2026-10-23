use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::fetch_three_basic_types::<{ lands::sub(Subtype::Mountain) }, { lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Swamp) }>(
            "{T}, Sacrifice this land: Search your library for a basic Mountain, Plains, or Swamp card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggers::cycling(mana("{R}{W}{B}"), "Cycling {R}{W}{B} ({R}{W}{B}, Discard this card: Draw a card.)"),
    ],
    ..oracle::SHATTERED_LANDSCAPE
};
