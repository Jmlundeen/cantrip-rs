use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+2 until end of turn." (CR 611.2a, 608.2b); convoke is the `convoke` field (CR 702.51).
pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effects::target_creature_gets::<2, 2>), ..oracle::GATHER_COURAGE };
