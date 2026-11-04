use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "Artifact creatures you control have double strike" (CR 702.4, 613.1f): a layer 6 ability of each artifact creature its
/// controller controls, this one included.
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        statics::your_artifact_creatures,
        statics::have_double_strike,
        "Artifact creatures you control have double strike.",
    )],
    ..oracle::PRIMARY_UNIMATRIX_ADJUNCT
};
