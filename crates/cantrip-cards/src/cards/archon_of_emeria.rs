use cantrip_api::*;
use cantrip_mechanics::replacements::{enter_tapped, opponents_enter_tapped};
use cantrip_oracle as oracle;

/// "Each player can't cast more than one spell each turn." (CR 101.2.)
pub static CARD: CardDef = CardDef {
    cast_limit: 1,
    replacements: &[opponents_enter_tapped::<{ enter_tapped::NONBASIC_LANDS }>("enters tapped")],
    ..oracle::ARCHON_OF_EMERIA
};
