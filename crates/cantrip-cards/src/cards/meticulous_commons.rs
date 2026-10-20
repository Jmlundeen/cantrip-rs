use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control_a_planeswalker),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Black]), "{T}: Add {W} or {B}.")],
    ..oracle::METICULOUS_COMMONS
};
