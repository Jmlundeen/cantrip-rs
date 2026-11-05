use cantrip_api::*;
use cantrip_mechanics::triggers::{BEGINNING_OF_YOUR_COMBAT, THIS_ENTERS};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::plus_one_counter_on_target, "When this creature enters, put a +1/+1 counter on another target creature you control.")
            .targets(&[targets::OTHER_CREATURE_YOU_CONTROL]),
        triggered(BEGINNING_OF_YOUR_COMBAT, effects::exile_first_target, "At the beginning of combat on your turn, exile up to one target card from a graveyard.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
    ],
    ..oracle::ASCENDANT_DUSTSPEAKER
};
