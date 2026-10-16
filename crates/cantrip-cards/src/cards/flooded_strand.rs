use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[lands::fetch_land::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Island) }>(
        "{T}, Pay 1 life, Sacrifice Flooded Strand: Search your library for a Plains or Island card, put it onto the battlefield, then shuffle.",
    )],
    ..oracle::FLOODED_STRAND
};
