use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_unless_two_or_more_other_lands),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Blue]), "{T}: Add {W} or {U}.")],
    ..oracle::DESERTED_BEACH
};
