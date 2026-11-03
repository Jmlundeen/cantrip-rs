use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}"),
            cantrip_mechanics::effects::this_gets::<1, 1>,
            "{1}: This creature gets +1/+1 until end of turn.",
        ),
    ],
    ..oracle::CARRION_ANTS
};
