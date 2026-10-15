use cantrip_api::*;
use cantrip_mechanics::{lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[lands::fetch_land::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Swamp) }>(
        "{T}, Pay 1 life, Sacrifice Marsh Flats: Search your library for a Plains or Swamp card, put it onto the battlefield, then shuffle.",
    )],
    ..oracle::MARSH_FLATS
};
