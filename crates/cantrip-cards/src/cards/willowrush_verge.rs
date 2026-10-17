use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Island) }>(
        Mana::U,
        Mana::G,
        "{T}: Add {U}.",
        "{T}: Add {G}. Activate only if you control a Forest or an Island.",
    ),
    ..oracle::WILLOWRUSH_VERGE
};
