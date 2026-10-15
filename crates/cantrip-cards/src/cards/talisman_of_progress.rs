use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        mana::tap_for_either_and_hurt(Color::White, Color::Blue, "{T}: Add {W} or {U}. This artifact deals 1 damage to you."),
    ],
    ..oracle::TALISMAN_OF_PROGRESS
};
