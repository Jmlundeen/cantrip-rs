use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// (Improvise is a characteristic of the card: `pay_with`.)
pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL], effects::counter_first_target_unless_pays::<3>), ..oracle::METALLIC_REBUKE };
