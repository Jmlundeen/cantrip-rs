use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP.and(SACRIFICE_THIS), add_any_color(3), "{T}, Sacrifice Black Lotus: Add three mana of any one color.")],
    ..oracle::BLACK_LOTUS
};
