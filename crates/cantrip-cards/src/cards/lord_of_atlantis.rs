use cantrip_api::*;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

/// "Other Merfolk get +1/+1 and have islandwalk." One static ability, one effect in two layers (CR 613.6): islandwalk in
/// layer 6 (CR 613.1f, 702.14), +1/+1 in layer 7c (CR 613.4c) to the same objects. "Other Merfolk" is every other
/// permanent with the creature type Merfolk, whoever controls it, as layer 4 left it (Arcane Adaptation or Conspiracy can
/// make a creature one; Turn to Frog makes one a Frog). A creature itself, the Lord loses this ability to Humility or Dress
/// Down before it can apply (CR 613.8a: its existence depends on them), so then no Merfolk gets anything. Merfolk is a word
/// of its text (CR 612).
const TEXT: &str = "Other Merfolk get +1/+1 and have islandwalk.";

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, statics::other_of_subtype::<{ lands::sub(Subtype::Merfolk) }>, statics::have_landwalk::<{ lands::sub(Subtype::Island) }>, TEXT),
        static_ability(Layer::PtModify, statics::other_of_subtype::<{ lands::sub(Subtype::Merfolk) }>, statics::plus_pt::<1, 1>, TEXT).continues(),
    ],
    ..oracle::LORD_OF_ATLANTIS
};
