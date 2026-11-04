use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{5}").and(TAP), effects::return_first_target_to_hand,
        "{5}, {T}: Return target enchantment card from your graveyard to your hand.")
        .targets(&[targets::ENCHANTMENT_CARD_IN_YOUR_GRAVEYARD])],
    ..oracle::SKULL_OF_ORM
};
