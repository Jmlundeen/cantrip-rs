use cantrip_api::*;
use cantrip_mechanics::learn;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, learn::learn, "When this creature dies, learn.")],
    ..oracle::EYETWITCH
};
