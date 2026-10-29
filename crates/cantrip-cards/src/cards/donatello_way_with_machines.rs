use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::ARTIFACT_YOU_CONTROL_ENTERS, effects::plus_one_counter_on_this, "Whenever an artifact you control enters, put a +1/+1 counter on Donatello.")],
    ..oracle::DONATELLO_WAY_WITH_MACHINES
};
