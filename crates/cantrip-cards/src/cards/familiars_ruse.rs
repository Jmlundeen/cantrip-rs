use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target),
    // As an additional cost to cast this spell, return a creature you control to its owner's hand (CR 118.8).
    additional_cost: Some(Cost::return_to_hand(1, targets::filters::creature)),
    ..oracle::FAMILIARS_RUSE
};
