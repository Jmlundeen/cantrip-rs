use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::ANY_COLOR_FOR_ONE,
        lands::SURVEIL_LAND,
    ],
    ..oracle::CONDUIT_PYLONS
};
