use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // As an additional cost to cast this spell, return a permanent you control to its owner's hand (CR 118.8).
    additional_cost: Some(Cost::return_to_hand(1, targets::filters::any)),
    ..oracle::FEAR_OF_ISOLATION
};
