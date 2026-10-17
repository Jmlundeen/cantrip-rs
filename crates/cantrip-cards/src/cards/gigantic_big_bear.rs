use cantrip_api::*;
use cantrip_oracle as oracle;

// Characteristics only: "This spell can't be countered" (CR 101.2), hexproof (CR 702.11) and haste (CR 702.10) are core keywords of the
// card (`CantBeCountered`, `Hexproof`, `Haste`), so nothing is left to write.
pub static CARD: CardDef = CardDef { ..oracle::GIGANTIC_BIG_BEAR };
