use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(
            cost("{5}, {T}"),
            add_each(&[Mana::W, Mana::U, Mana::B, Mana::R, Mana::G]),
            "{5}, {T}: Add {W}{U}{B}{R}{G}.",
        ),
    ],
    ..oracle::CRYSTAL_QUARRY
};
