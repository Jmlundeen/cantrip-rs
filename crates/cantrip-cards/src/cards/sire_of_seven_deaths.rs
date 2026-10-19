use cantrip_api::*;
use cantrip_mechanics::triggers::ward_life;
use cantrip_oracle as oracle;

/// "Ward\u{2014}Pay 7 life" (CR 702.21a): the other six keywords are core keywords of the generated constant.
pub static CARD: CardDef = CardDef {
    abilities: &[ward_life::<7>("Ward\u{2014}Pay 7 life.")],
    ..oracle::SIRE_OF_SEVEN_DEATHS
};
