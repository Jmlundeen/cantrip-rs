use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ANY], |g, item| effects::damage_target_and_gain_life(g, item, 3, 3)), ..oracle::LIGHTNING_HELIX };
