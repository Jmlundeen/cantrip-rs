use cantrip_api::*;
use cantrip_mechanics::{prepare, statics::{itself_with_threshold, plus_pt}};
use cantrip_oracle as oracle;

/// Enters prepared (CR 722.3a); threshold modifies power and toughness in layer 7c (CR 613.4c).
/// Omit Variables is the identical prepare spell printed on Paradox Shaper.
pub static CARD: CardDef = CardDef {
    enters: Some(prepare::enters_prepared),
    prepare: Some(&super::paradox_shaper::OMIT_VARIABLES),
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_threshold,
        plus_pt::<1, 1>,
        "Threshold — This creature gets +1/+1 as long as there are seven or more cards in your graveyard.",
    )],
    ..oracle::VOID_EXTRAPOLATOR
};
