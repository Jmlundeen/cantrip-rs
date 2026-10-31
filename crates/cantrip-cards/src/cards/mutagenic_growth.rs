use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+2 until end of turn." (CR 611.2a, 608.2b); the {G/P} is paid with {G} or 2 life by the
/// engine (CR 107.4f).
pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effects::target_creature_gets::<2, 2>), ..oracle::MUTAGENIC_GROWTH };
