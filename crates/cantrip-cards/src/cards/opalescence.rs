use cantrip_api::*;
use cantrip_mechanics::statics::{base_mana_value, creature_too};
use cantrip_oracle as oracle;

/// "Each other non-Aura enchantment is a creature in addition to its other types and has base power and base toughness
/// each equal to its mana value." One ability, one effect in two layers (CR 613.6): the type is added in layer 4 (CR
/// 613.1d, 205.1b: it keeps its other types), and the base power and toughness is set in layer 7b (CR 613.4b), to the
/// permanents the layer-4 part made creatures, even once Humility has taken this ability away (the rulings); effects that
/// modify power and toughness and counters apply after it (layer 7c). "Other": not Opalescence itself. Aura is an
/// enchantment type (CR 303.4), no word of the text a text-changing effect can change (CR 612.2).
const TEXT: &str =
    "Each other non-Aura enchantment is a creature in addition to its other types and has base power and base toughness each equal to its mana value.";

/// "Each other non-Aura enchantment", as the layers before have it.
fn other_non_aura_enchantment(_: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    affected != source && so_far.is(CardType::Enchantment) && !so_far.has_subtype(Subtype::Aura)
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Type, other_non_aura_enchantment, creature_too, TEXT),
        static_ability_for_each(Layer::PtSet, other_non_aura_enchantment, base_mana_value, TEXT).continues(),
    ],
    ..oracle::OPALESCENCE
};
