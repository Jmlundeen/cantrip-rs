use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "Enchanted creature ... is a ... Skeleton creature" (CR 205.1a, 613.1d, layer 4): its card types are replaced by
/// creature alone and its subtypes by Skeleton, the subtypes of the types it loses going with them ("It loses all other
/// ... card types, and creature types"). Its supertypes stay. Skeleton is a word of this card's text (CR 612).
fn skeleton_creature(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.types = TypeSet::of(&[CardType::Creature]);
    chars.subtypes = SubtypeSet::of(&[view.words(source).subtype(Subtype::Skeleton)]);
}

/// "Enchanted creature ... is a black ... creature" (CR 105.3, layer 5): black and no other color.
fn black(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.colors = view.words(source).colors(ColorSet::of(Color::Black));
}

/// "... with base power and toughness 1/1" (CR 613.4b, layer 7b): counters and effects that modify power and toughness
/// still apply after it.
fn base_one_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(1);
    chars.toughness = Some(1);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Type, attached_object, skeleton_creature, "Enchanted creature is a Skeleton creature."),
        static_ability(Layer::Color, attached_object, black, "Enchanted creature is black."),
        loses_all_abilities(attached_object, "Enchanted creature loses all abilities."),
        static_ability(Layer::PtSet, attached_object, base_one_one, "Enchanted creature has base power and toughness 1/1."),
    ],
    ..oracle::EATEN_BY_PIRANHAS
};
