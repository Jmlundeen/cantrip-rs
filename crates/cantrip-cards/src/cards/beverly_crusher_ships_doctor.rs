use cantrip_api::*;
use cantrip_mechanics::seek_out::seek_out;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[seek_out(
        effects::plus_one_counter_on_each_target,
        &[targets::CREATURE.between(0, 3)],
        "Seek out — Whenever you draw your second card each turn, put a +1/+1 counter on each of up to three target creatures.",
    )],
    ..oracle::BEVERLY_CRUSHER_SHIPS_DOCTOR
};
