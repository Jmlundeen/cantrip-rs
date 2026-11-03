use cantrip_api::*;
use cantrip_mechanics::replacements::{enter_tapped, opponents_enter_tapped};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    replacements: &[opponents_enter_tapped::<{ enter_tapped::CREATURES | enter_tapped::ARTIFACTS | enter_tapped::LANDS }>(
        "Artifacts, creatures, and lands your opponents control enter tapped.",
    )],
    ..oracle::KISMET
};
