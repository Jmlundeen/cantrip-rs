use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Island) }, { lands::sub(Subtype::Swamp) }>(
        Mana::U,
        Mana::B,
        "{T}: Add {U}.",
        "{T}: Add {B}. Activate only if you control an Island or a Swamp.",
    ),
    ..oracle::GLOOMLAKE_VERGE
};
