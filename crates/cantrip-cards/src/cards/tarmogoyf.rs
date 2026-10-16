use cantrip_api::*;
use cantrip_mechanics::goyf;
use cantrip_oracle as oracle;

/// Printed */1+*: with its ability gone the star is worth 0 (CR 208.2a, 208.5), so a 0/1.
pub static CARD: CardDef = CardDef { statics: &[goyf::SIZE], power: Some(0), toughness: Some(1), ..oracle::TARMOGOYF };
