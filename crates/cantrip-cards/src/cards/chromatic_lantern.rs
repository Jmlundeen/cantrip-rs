use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    // "{T}: Add one mana of any color.", the mana ability the lands are given: the land is its source (CR 113.6).
    delayed: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    // "Lands you control have ..." (CR 613.1f, layer 6): granted after any effect of layer 4 that set a land's subtype took
    // the abilities of its rules text, so a land Blood Moon makes a Mountain keeps it (CR 305.7: "this doesn't remove any
    // abilities that were granted to the land by other effects"). The lands keep their land types and other abilities (the
    // ruling).
    statics: &[static_ability(
        Layer::Ability,
        statics::your_lands,
        statics::grant_delayed_ability,
        "Lands you control have \"{T}: Add one mana of any color.\"",
    )],
    ..oracle::CHROMATIC_LANTERN
};
