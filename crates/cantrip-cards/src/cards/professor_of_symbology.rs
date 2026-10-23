use cantrip_api::*;
use cantrip_mechanics::learn;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, learn::learn, "When this creature enters, learn.")],
    ..oracle::PROFESSOR_OF_SYMBOLOGY
};
