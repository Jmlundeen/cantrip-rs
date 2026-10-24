use cantrip_api::*;
use cantrip_mechanics::{prepare, statics};
use cantrip_oracle as oracle;

/// "Each creature you control with a +1/+1 counter on it has vigilance" (CR 613.1f, 702.20).
/// Its Seed Suture prepare spell is identical to Blossom-Blessed Angel's (CR 722).
pub static CARD: CardDef = CardDef {
    enters: Some(prepare::enters_prepared),
    prepare: Some(&super::blossom_blessed_angel::SEED_SUTURE),
    statics: &[static_ability(
        Layer::Ability,
        statics::your_creatures_with_plus_one_counters,
        statics::have_vigilance,
        "Each creature you control with a +1/+1 counter on it has vigilance.",
    )],
    ..oracle::VIGORBLOOM_VANGUARD
};
