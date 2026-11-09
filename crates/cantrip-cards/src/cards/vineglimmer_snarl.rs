use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_one_of(&[Color::Green, Color::Blue]),
            "{T}: Add {G} or {U}.",
        ),
    ],
    enters: Some(lands::enters_tapped_unless_you_reveal::<{ lands::sub(Subtype::Forest) }, { lands::sub(Subtype::Island) }>),
    ..oracle::VINEGLIMMER_SNARL
};
