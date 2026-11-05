use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// Menace is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::another_creature_or_artifact_you_control_dies),
        effects::plus_one_counter_on_this,
        "Whenever another creature or artifact you control dies, put a +1/+1 counter on Judge Magister Gabranth.",
    )],
    ..oracle::JUDGE_MAGISTER_GABRANTH
};
