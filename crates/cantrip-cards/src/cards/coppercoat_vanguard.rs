use cantrip_api::*;
use cantrip_mechanics::{lands, statics, triggers::ward};
use cantrip_oracle as oracle;

/// "Each other Human you control gets +1/+0 and has ward {1}." One static ability in two layers (CR 613.6): ward in layer 6
/// (CR 613.1f, 702.21) and +1/+0 in layer 7c (CR 613.4c), to the same objects: every other permanent its controller controls
/// that is a Human as the layers have it so far, the type read through the words of this card's text (CR 612). Each
/// affected Human has a ward ability of its own (CR 113.2c, `statics::grant_delayed_ability`).
pub static CARD: CardDef = CardDef {
    delayed: &[ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[
        static_ability(Layer::Ability, statics::other_of_subtype_you_control::<{ lands::sub(Subtype::Human) }>, statics::grant_delayed_ability, "Each other Human you control has ward {1}."),
        static_ability(Layer::PtModify, statics::other_of_subtype_you_control::<{ lands::sub(Subtype::Human) }>, statics::plus_pt::<1, 0>, "Each other Human you control gets +1/+0.").continues(),
    ],
    ..oracle::COPPERCOAT_VANGUARD
};
