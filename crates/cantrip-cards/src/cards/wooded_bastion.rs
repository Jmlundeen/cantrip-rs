use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        cantrip_mechanics::lands::filter_two::<{ ColorSet::of(Color::Green).0 | ColorSet::of(Color::White).0 }>(
            cost("{G/W}, {T}"),
            "{G/W}, {T}: Add {G}{G}, {G}{W}, or {W}{W}.",
        ),
    ],
    ..oracle::WOODED_BASTION
};
