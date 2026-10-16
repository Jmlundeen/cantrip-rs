use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Creatures can't attack you unless their controller pays {2} for each creature they control that's attacking you."
    // (CR 508.1d, 508.1h)
    attack_costs: &[statics::attacking_you_costs::<2>],
    ..oracle::GHOSTLY_PRISON
};
