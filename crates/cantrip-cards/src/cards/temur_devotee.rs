use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        cost("{1}"),
        add_one_of(&[Color::Green, Color::Blue, Color::Red]),
        "{1}: Add {G}, {U}, or {R}. Activate only once each turn.",
    )
    .once_each_turn()],
    ..oracle::TEMUR_DEVOTEE
};
