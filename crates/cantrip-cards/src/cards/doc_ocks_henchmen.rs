use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, effects::this_connives, "Whenever this creature attacks, it connives.")],
    ..oracle::DOC_OCKS_HENCHMEN
};
