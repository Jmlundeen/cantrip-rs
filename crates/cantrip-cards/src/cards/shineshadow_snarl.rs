use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_one_of(&[Color::White, Color::Black]),
        "{T}: Add {W} or {B}.",
    )],
    enters: Some(lands::enters_tapped_unless_you_reveal::<{ lands::sub(Subtype::Plains) }, { lands::sub(Subtype::Swamp) }>),
    ..oracle::SHINESHADOW_SNARL
};
