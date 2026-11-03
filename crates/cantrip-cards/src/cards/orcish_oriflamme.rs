use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        statics::attacking_creatures_you_control,
        statics::plus_pt::<1, 0>,
        "Attacking creatures you control get +1/+0.",
    )],
    ..oracle::ORCISH_ORIFLAMME
};
