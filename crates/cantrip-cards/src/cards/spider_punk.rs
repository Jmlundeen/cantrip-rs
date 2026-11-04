use cantrip_api::*;
use cantrip_mechanics::statics::{cant_be_countered, damage_cant_be_prevented, other_of_subtype_you_control, spells};
use cantrip_mechanics::{lands, riot};
use cantrip_oracle as oracle;

/// "... and abilities can't be countered." (CR 101.2, 701.6a): every activated and triggered ability on the stack, whoever
/// controls it.
fn abilities(_: &dyn Query, _: &Asker, _: &StackItem) -> bool {
    true
}

// Riot is printed (`oracle::SPIDER_PUNK.riot`), and the core makes its choice.
pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            other_of_subtype_you_control::<{ lands::sub(Subtype::Spider) }>,
            riot::have_riot,
            "Other Spiders you control have riot.",
        ),
        static_ability(Layer::Ability, spells, cant_be_countered, "Spells can't be countered.").everywhere(),
    ],
    uncounterable_abilities: &[abilities],
    unpreventable_damage: &[damage_cant_be_prevented],
    ..oracle::SPIDER_PUNK
};
