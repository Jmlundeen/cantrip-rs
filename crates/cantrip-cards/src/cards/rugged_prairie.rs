use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Red).0 | ColorSet::of(Color::White).0 }>(
            cost("{R/W}, {T}"),
            "{R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}.",
        ),
    ],
    ..oracle::RUGGED_PRAIRIE
};
