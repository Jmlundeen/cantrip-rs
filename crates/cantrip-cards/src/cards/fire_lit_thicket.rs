use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Red).0 | ColorSet::of(Color::Green).0 }>(
            cost("{R/G}, {T}"),
            "{R/G}, {T}: Add {R}{R}, {R}{G}, or {G}{G}.",
        ),
    ],
    ..oracle::FIRE_LIT_THICKET
};
