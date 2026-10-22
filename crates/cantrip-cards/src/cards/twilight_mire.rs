use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Black).0 | ColorSet::of(Color::Green).0 }>(
            cost("{B/G}, {T}"),
            "{B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G}.",
        ),
    ],
    ..oracle::TWILIGHT_MIRE
};
