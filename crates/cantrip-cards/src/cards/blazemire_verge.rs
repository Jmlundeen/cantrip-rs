use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Mountain) }>(
        Mana::B,
        Mana::R,
        "{T}: Add {B}.",
        "{T}: Add {R}. Activate only if you control a Swamp or a Mountain.",
    ),
    ..oracle::BLAZEMIRE_VERGE
};
