use cantrip_api::*;
use cantrip_mechanics::{effects::plus_one_counter_on_this, triggers::YOU_FACE_A_DILEMMA};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_FACE_A_DILEMMA,
        plus_one_counter_on_this,
        "Whenever you face a dilemma, put a +1/+1 counter on this creature. (You face a dilemma as you choose one or more modes for a spell or ability.)",
    )],
    ..oracle::MELKOTIAN_ADJUDICATOR
};
