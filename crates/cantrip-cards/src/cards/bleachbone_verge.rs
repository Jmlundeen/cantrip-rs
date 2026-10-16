use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Swamp) }>(
        Mana::B,
        Mana::W,
        "{T}: Add {B}.",
        "{T}: Add {W}. Activate only if you control a Plains or a Swamp.",
    ),
    ..oracle::BLEACHBONE_VERGE
};
