use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Destroy target artifact." (CR 701.8a, 608.2b).
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT], effects::destroy_first_target),
    ..oracle::SHATTER
};
