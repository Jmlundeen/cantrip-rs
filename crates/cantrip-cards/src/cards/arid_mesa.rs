use cantrip_api::*;
use cantrip_mechanics::{lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[lands::fetch_land::<{ lands::sub(Subtype::Mountain) }, { lands::sub(Subtype::Plains) }>(
        "{T}, Pay 1 life, Sacrifice Arid Mesa: Search your library for a Mountain or Plains card, put it onto the battlefield, then shuffle.",
    )],
    ..oracle::ARID_MESA
};
