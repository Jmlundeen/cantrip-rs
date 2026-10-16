use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Mountain) }, { lands::sub(Subtype::Forest) }>(
        Mana::R,
        Mana::G,
        "{T}: Add {R}.",
        "{T}: Add {G}. Activate only if you control a Mountain or a Forest.",
    ),
    ..oracle::THORNSPIRE_VERGE
};
