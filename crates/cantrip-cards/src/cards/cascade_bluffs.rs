use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Red).0 }>(
            cost("{U/R}, {T}"),
            "{U/R}, {T}: Add {U}{U}, {U}{R}, or {R}{R}.",
        ),
    ],
    ..oracle::CASCADE_BLUFFS
};
