use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Forest) }>(
        Mana::G,
        Mana::B,
        "{T}: Add {G}.",
        "{T}: Add {B}. Activate only if you control a Swamp or a Forest.",
    ),
    ..oracle::WASTEWOOD_VERGE
};
