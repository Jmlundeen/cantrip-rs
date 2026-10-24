use cantrip_api::*;
use cantrip_mechanics::{enduring, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "{T}: Add one mana of any color.", the mana ability the creatures are given: the creature is its source (CR 113.7).
    delayed: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    // "Creatures you control have ..." (CR 613.1f, layer 6): Enduring Vitality itself included while it is a creature.
    statics: &[static_ability(
        Layer::Ability,
        statics::your_creatures,
        statics::grant_delayed_ability,
        "Creatures you control have \"{T}: Add one mana of any color.\"",
    )],
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, enduring::dies_as_a_creature), enduring::return_as_enchantment, "When Enduring Vitality dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)")],
    ..oracle::ENDURING_VITALITY
};
