use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, statics::base_pt, targets};
use cantrip_oracle as oracle;

// "Enchanted creature is a Treefolk with base power and toughness 0/4 and loses all abilities.": one static ability, an
// effect in layers 4, 6 and 7b, all at the time this Aura became attached (CR 613.7e).

/// "... is a Treefolk" (CR 205.1a, layer 4): its creature types become Treefolk alone; its card types, its other subtypes
/// and its colors stay. A changeling's every creature type is overwritten (the rulings). Treefolk is a word of this card's
/// rules text (CR 612).
fn treefolk(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let kept = chars.subtypes.iter().filter(|t| !t.is_creature_type()).fold(SubtypeSet::NONE, SubtypeSet::with);
    chars.subtypes = kept.with(view.words(source).subtype(Subtype::Treefolk));
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Type, attached_object, treefolk, "Enchanted creature is a Treefolk."),
        static_ability(Layer::PtSet, attached_object, base_pt::<0, 4>, "Enchanted creature has base power and toughness 0/4."),
        attached_loses_all_abilities("Enchanted creature loses all abilities."),
    ],
    ..oracle::LIGNIFY
};
