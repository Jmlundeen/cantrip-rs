use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(cost("{3}, {T}"), tokens::create_a_treasure, "{3}, {T}: Create a Treasure token."),
    ],
    ..oracle::SAVVY_PROFITEER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
