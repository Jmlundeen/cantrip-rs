use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Black).0 | ColorSet::of(Color::Red).0 }>(
            cost("{B/R}, {T}"),
            "{B/R}, {T}: Add {B}{B}, {B}{R}, or {R}{R}.",
        ),
    ],
    ..oracle::GRAVEN_CAIRNS
};
