use cantrip_api::*;
use cantrip_mechanics::{lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::R }, { lands::U }, { lands::sub(Subtype::Island) }, { lands::sub(Subtype::Mountain) }>(
        "{T}: Add {R}.",
        "{T}: Add {U}. Activate only if you control an Island or a Mountain.",
    ),
    ..oracle::RIVERPYRE_VERGE
};
