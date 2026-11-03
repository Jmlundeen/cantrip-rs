use cantrip_api::*;
use cantrip_mechanics::statics::{self, base_pt, creature_type_alone};
use cantrip_mechanics::{attach::{attached_object, tap_attached}, effects, lands, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

// "Enchanted creature loses all abilities and is a Citizen with base power and toughness 1/1 and "{T}: Add {C}" named Humble
// Merchant. (It loses all other creature types and names.)": one static ability, an effect in each of four layers, all at the
// time this Aura became attached (CR 613.7e).

/// "... named Humble Merchant" (CR 612.8, layer 3): a text-changing effect that sets the name; the creature has no other name.
fn humble_merchant(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.name = "Humble Merchant";
    chars.name_set = true;
}

/// "When this Aura enters, tap enchanted creature and remove all counters from it." The permanent the Aura is attached to when
/// the ability resolves is tapped, then loses every counter of every kind (CR 701.26a, 122.1, 303.4m); nothing happens if it is
/// attached to nothing (CR 608.2h).
fn tap_and_remove_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = tap_attached(g, item) {
        effects::remove_all_counters(g, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_AN_OPPONENT_CONTROLS),
    abilities: &[triggered(
        THIS_ENTERS,
        tap_and_remove_counters,
        "When this Aura enters, tap enchanted creature and remove all counters from it.",
    )],
    // "{T}: Add {C}", the mana ability the creature is given: the creature is its source (CR 113.7).
    delayed: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")],
    statics: &[
        static_ability(Layer::Text, attached_object, humble_merchant, "Enchanted creature is named Humble Merchant."),
        static_ability(Layer::Type, attached_object, creature_type_alone::<{ lands::sub(Subtype::Citizen) }>, "Enchanted creature is a Citizen.").continues(),
        loses_all_abilities(attached_object, "Enchanted creature loses all abilities.").continues(),
        static_ability(Layer::Ability, attached_object, statics::grant_delayed_ability, "Enchanted creature has \"{T}: Add {C}.\""),
        static_ability(Layer::PtSet, attached_object, base_pt::<1, 1>, "Enchanted creature has base power and toughness 1/1.").continues(),
    ],
    ..oracle::HONEST_WORK
};
