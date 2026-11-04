use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_BECOMES_TAPPED;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_BECOMES_TAPPED, effects::untap_targets, "Whenever this creature becomes tapped, untap another target permanent. This ability triggers only once each turn.")
        .targets(&[targets::OTHER_PERMANENT])
        .triggers_each_turn(1)],
    ..oracle::NANOFORM_SENTINEL
};
