use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Landfall — Whenever a land you control enters, put two +1/+1 counters on target creature you control" (CR 122.1,
/// 603.6a): the target is chosen as the trigger is put on the stack and judged again as it resolves (CR 603.3d, 608.2b).
/// Trample is a core keyword in the characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        effects::two_plus_one_counters_on_target,
        "Landfall \u{2014} Whenever a land you control enters, put two +1/+1 counters on target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::SYNDICATE_LIQUIDATORS
};
