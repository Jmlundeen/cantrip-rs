use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{affinity, effects, lands, targets};
use cantrip_oracle as oracle;

/// "Affinity for Vulcans" (CR 702.41a): this spell costs {1} less to cast for each permanent with the Vulcan subtype
/// its controller controls, the type read through the spell's own words (CR 612.2). The landfall ability is the shared
/// recipe: the target is chosen as the trigger is put on the stack and judged again as it resolves (CR 603.3d, 608.2b).
pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_creature_type::<{ lands::sub(Subtype::Vulcan) }>],
    abilities: &[triggered(
        LANDFALL,
        effects::two_plus_one_counters_on_target,
        "Landfall \u{2014} Whenever a land you control enters, put two +1/+1 counters on target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::AMANDA_GRAYSON_LOGICS_LINK
};
