use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[lands::fetch_land::<{ lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Mountain) }>(
        "{T}, Pay 1 life, Sacrifice Bloodstained Mire: Search your library for a Swamp or Mountain card, put it onto the battlefield, then shuffle.",
    )],
    ..oracle::BLOODSTAINED_MIRE
};
