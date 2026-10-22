use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0 }>(
            cost("{U/B}, {T}"),
            "{U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}.",
        ),
    ],
    ..oracle::SUNKEN_RUINS
};
