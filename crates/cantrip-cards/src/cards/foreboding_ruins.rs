use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_one_of(&[Color::Black, Color::Red]),
            "{T}: Add {B} or {R}.",
        ),
    ],
    enters: Some(lands::enters_tapped_unless_you_reveal::<{ lands::sub(Subtype::Swamp) }, { lands::sub(Subtype::Mountain) }>),
    ..oracle::FOREBODING_RUINS
};
