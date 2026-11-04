use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{T}: Target creature you control gets +1/+1 and gains haste until end of turn. Activate only as a sorcery." The target
/// is chosen as the ability is activated and judged again as it resolves (CR 608.2b); it may be the Turbodrone itself, which
/// is tapped for the cost but is still a creature you control.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        effects::target_creature_gets_and_gains_haste::<1, 1>,
        "{T}: Target creature you control gets +1/+1 and gains haste until end of turn. Activate only as a sorcery.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])
    .sorcery_speed()],
    ..oracle::KAVARON_TURBODRONE
};
