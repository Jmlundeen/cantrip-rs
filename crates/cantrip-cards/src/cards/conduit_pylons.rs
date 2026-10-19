use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(cost("{1}, {T}"), add_any_color(1), "{1}, {T}: Add one mana of any color."),
        lands::SURVEIL_LAND,
    ],
    ..oracle::CONDUIT_PYLONS
};
