use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        effects::this_gets::<2, 0>,
        "Landfall \u{2014} Whenever a land you control enters, this creature gets +2/+0 until end of turn.",
    )],
    ..oracle::REMNANT_ELEMENTAL
};
