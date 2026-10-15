use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add {B}, {R}, or {G}.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(cantrip_mechanics::mana::add_chosen),
            any_color: true,
            colors: ColorSet(0b11100),
            adds_any: 1,
            ..ActivatedDef::BLANK
        }),
        triggers::EXALTED,
    ],
    ..oracle::IGNOBLE_HIERARCH
};
