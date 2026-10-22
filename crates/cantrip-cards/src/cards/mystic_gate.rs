use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::White).0 | ColorSet::of(Color::Blue).0 }>(
            cost("{W/U}, {T}"),
            "{W/U}, {T}: Add {W}{W}, {W}{U}, or {U}{U}.",
        ),
    ],
    ..oracle::MYSTIC_GATE
};
