use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "This land enters tapped. {T}: Add {G}, {W}, or {U}." (CR 614.1d, 605.1a): the {T} ability is a mana ability offering
/// green, white or blue and no other color.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_one_of(&[Color::Green, Color::White, Color::Blue]),
        "{T}: Add {G}, {W}, or {U}.",
    )],
    enters: Some(mana::enters_tapped),
    ..oracle::SEASIDE_CITADEL
};
