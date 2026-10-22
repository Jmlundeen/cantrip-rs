use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Green).0 | ColorSet::of(Color::Blue).0 }>(
            cost("{G/U}, {T}"),
            "{G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}.",
        ),
    ],
    ..oracle::FLOODED_GROVE
};
