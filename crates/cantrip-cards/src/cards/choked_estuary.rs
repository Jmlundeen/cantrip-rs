use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_one_of(&[Color::Blue, Color::Black]),
            "{T}: Add {U} or {B}.",
        ),
    ],
    enters: Some(lands::enters_tapped_unless_you_reveal::<{ lands::sub(Subtype::Island) }, { lands::sub(Subtype::Swamp) }>),
    ..oracle::CHOKED_ESTUARY
};
