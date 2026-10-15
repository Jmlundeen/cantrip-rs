use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add {G}, {W}, or {U}.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(mana::add_chosen),
            any_color: true,
            colors: ColorSet(0b10011),
            adds_any: 1,
            ..ActivatedDef::BLANK
        }),
        triggers::EXALTED,
    ],
    ..oracle::NOBLE_HIERARCH
};
