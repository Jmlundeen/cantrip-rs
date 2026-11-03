use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[effects::REMOVE_PLUS_ONE_TO_DEAL_ONE],
    enters: Some(effects::enters_with_plus_one_counters::<3>),
    ..oracle::TRISKELION
};
