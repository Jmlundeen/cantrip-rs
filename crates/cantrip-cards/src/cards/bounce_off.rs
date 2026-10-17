use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Return target creature or Vehicle to its owner's hand."
pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_OR_VEHICLE], effects::return_first_target_to_hand), ..oracle::BOUNCE_OFF };
