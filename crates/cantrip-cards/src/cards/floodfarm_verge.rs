use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Island) }>(
        Mana::W,
        Mana::U,
        "{T}: Add {W}.",
        "{T}: Add {U}. Activate only if you control a Plains or an Island.",
    ),
    ..oracle::FLOODFARM_VERGE
};
