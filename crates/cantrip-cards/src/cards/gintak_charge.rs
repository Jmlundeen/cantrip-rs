use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Gin'tak Charge deals 2 damage to any target." (CR 115.1, 120.3): one target from the whole set a spell may take - either
/// player, or any creature, planeswalker or battle on either side - gaining 2 damage as the spell resolves, the amount fixed
/// by the card's text and scaled by nothing.
pub static CARD: CardDef = CardDef { spell: spell(&[targets::ANY], effects::damage_first_target::<2>), ..oracle::GINTAK_CHARGE };
