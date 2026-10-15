use cantrip_api::*;
use cantrip_mechanics::{lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[lands::fetch_land::<{ lands::sub(Subtype::Island) }, { lands::sub(Subtype::Mountain) }>(
        "{T}, Pay 1 life, Sacrifice Scalding Tarn: Search your library for a Island or Mountain card, put it onto the battlefield, then shuffle.",
    )],
    ..oracle::SCALDING_TARN
};
