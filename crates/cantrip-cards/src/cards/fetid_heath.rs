use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::White).0 | ColorSet::of(Color::Black).0 }>(
            cost("{W/B}, {T}"),
            "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}.",
        ),
    ],
    ..oracle::FETID_HEATH
};
