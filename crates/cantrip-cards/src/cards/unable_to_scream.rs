use cantrip_api::*;
use cantrip_mechanics::statics::base_pt;
use cantrip_mechanics::{attach::attached_object, attach::attached_to_it, targets};
use cantrip_oracle as oracle;

// "Enchanted creature loses all abilities and is a Toy artifact creature with base power and toughness 0/2 in addition to its
// other types. As long as enchanted creature is face down, it can't be turned face up.": the first sentence is one static
// ability, an effect in layers 4, 6 and 7b, all at the time this Aura became attached (CR 613.7e). The second is a rule of the
// Aura's own (`face_up_bans`, CR 708.7), no ability of the creature, so it holds when the creature loses all abilities.

/// "... is a Toy artifact creature ... in addition to its other types" (CR 205.1b, 613.1d, layer 4): it gains the Artifact type
/// and the subtype Toy and keeps its other card types and subtypes. Toy is a word of this card's rules text (CR 612).
fn toy_artifact(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Artifact).with(CardType::Creature);
    chars.subtypes = chars.subtypes.with(view.words(source).subtype(Subtype::Toy));
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Type, attached_object, toy_artifact, "Enchanted creature is a Toy artifact creature in addition to its other types."),
        loses_all_abilities(attached_object, "Enchanted creature loses all abilities."),
        static_ability(Layer::PtSet, attached_object, base_pt::<0, 2>, "Enchanted creature has base power and toughness 0/2."),
    ],
    face_up_bans: &[attached_to_it],
    ..oracle::UNABLE_TO_SCREAM
};
