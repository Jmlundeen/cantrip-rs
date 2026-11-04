use cantrip_api::*;
use cantrip_mechanics::affinity;
use cantrip_oracle as oracle;

// Double strike and vigilance are core keywords, set by the content JSON.
pub static CARD: CardDef = CardDef {
    cost_changes: &[affinity::spells_you_cast_have_affinity_for_artifacts],
    ..oracle::SAMI_WILDCAT_CAPTAIN
};
