use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, cantrip_mechanics::statics::your_artifact_creatures, cantrip_mechanics::statics::plus_pt::<2, 2>, "Artifact creatures you control get +2/+2.")],
    ..oracle::TEMPERED_STEEL
};
