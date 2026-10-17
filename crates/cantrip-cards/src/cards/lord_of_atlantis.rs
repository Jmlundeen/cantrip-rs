use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "Other Merfolk get +1/+1 and have islandwalk." One static ability, one effect in two layers (CR 613.6): islandwalk in
/// layer 6 (CR 613.1f, 702.14), +1/+1 in layer 7c (CR 613.4c) to the same objects. "Other Merfolk" is every other
/// permanent with the creature type Merfolk, whoever controls it, as layer 4 left it (Arcane Adaptation or Conspiracy can
/// make a creature one; Turn to Frog makes one a Frog). A creature itself, the Lord loses this ability to Humility or Dress
/// Down before it can apply (CR 613.8a: its existence depends on them), so then no Merfolk gets anything. Merfolk is a word
/// of its text (CR 612).
const TEXT: &str = "Other Merfolk get +1/+1 and have islandwalk.";

fn other_merfolk(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    affected != source && so_far.has_subtype(q.words(source).subtype(Subtype::Merfolk))
}

/// "... have islandwalk" (layer 6): the landwalk of the land type the Lord's text names, Island unless a text-changing
/// effect replaced it with another basic land type (CR 612.2: Trait Doctoring's Swamp makes it swampwalk).
fn islandwalk(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(walk) = Keyword::landwalk(q.words(source).subtype(Subtype::Island)) {
        chars.keywords = chars.keywords.with(walk);
    }
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, other_merfolk, islandwalk, TEXT),
        static_ability(Layer::PtModify, other_merfolk, plus_pt::<1, 1>, TEXT).continues(),
    ],
    ..oracle::LORD_OF_ATLANTIS
};
