//! Baird, Steward of Argive. Vigilance is a core keyword, read from its characteristics.

use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Creatures can't attack you or planeswalkers you control unless their controller pays {1} for each of those
    // creatures." (CR 508.1d, 508.1h)
    attack_costs: &[statics::attacking_you_or_your_planeswalkers_costs::<1>],
    ..oracle::BAIRD_STEWARD_OF_ARGIVE
};
