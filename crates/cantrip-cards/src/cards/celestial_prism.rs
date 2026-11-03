use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{2}, {T}: Add one mana of any color." (CR 605.1a).
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            cost("{2}").and(TAP),
            add_any_color(1),
            "{2}, {T}: Add one mana of any color.",
        ),
    ],
    ..oracle::CELESTIAL_PRISM
};
