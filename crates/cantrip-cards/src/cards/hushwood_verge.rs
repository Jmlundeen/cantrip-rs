use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::verge::<{ lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Plains) }>(
        Mana::G,
        Mana::W,
        "{T}: Add {G}.",
        "{T}: Add {W}. Activate only if you control a Forest or a Plains.",
    ),
    ..oracle::HUSHWOOD_VERGE
};
