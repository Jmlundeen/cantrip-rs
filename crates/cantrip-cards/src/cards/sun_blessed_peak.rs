use cantrip_api::*;
use cantrip_mechanics::{effects, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Red, Color::White]), "{T}: Add {R} or {W}."),
        ability(cost("{4}, {T}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{4}, {T}, Sacrifice this land: Draw a card."),
    ],
    ..oracle::SUN_BLESSED_PEAK
};
