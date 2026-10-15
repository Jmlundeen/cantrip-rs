use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_unless_two_or_more_other_lands),
    abilities: &[mana::tap_for_either(Color::White, Color::Black, "{T}: Add {W} or {B}.")],
    ..oracle::SHATTERED_SANCTUM
};
