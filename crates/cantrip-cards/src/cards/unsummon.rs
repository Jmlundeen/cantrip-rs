use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Return target creature to its owner's hand." (CR 400.3, 608.2b)
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::return_first_target_to_hand),
    ..oracle::UNSUMMON
};
