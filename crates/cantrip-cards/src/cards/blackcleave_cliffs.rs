use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_unless_two_or_fewer_other_lands),
    abilities: &[mana::tap_for_either(Color::Black, Color::Red, "{T}: Add {B} or {R}.")],
    ..oracle::BLACKCLEAVE_CLIFFS
};

