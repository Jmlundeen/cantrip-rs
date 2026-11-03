use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{R}"),
        effects::this_gets::<0, 1>,
        "{R}: This creature gets +0/+1 until end of turn.",
    )],
    ..oracle::GRANITE_GARGOYLE
};
