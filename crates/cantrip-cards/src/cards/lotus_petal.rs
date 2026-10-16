use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP.and(SACRIFICE_THIS), add_any_color(1), "{T}, Sacrifice Lotus Petal: Add one mana of any color.")],
    ..oracle::LOTUS_PETAL
};
