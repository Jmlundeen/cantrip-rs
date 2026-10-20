use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control_a_planeswalker),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Blue, Color::Red]), "{T}: Add {U} or {R}.")],
    ..oracle::INNOVATIVE_COMMONS
};
