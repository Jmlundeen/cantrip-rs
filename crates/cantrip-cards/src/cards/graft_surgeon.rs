use cantrip_api::*;
use cantrip_mechanics::effects::put_its_counters_on_target;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::effects::enters_with_a_counter::<{ CounterKind::PlusOne as usize }>),
    abilities: &[triggered(
        THIS_DIES,
        put_its_counters_on_target,
        "When this creature dies, put its counters on up to one target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL.optional()])],
    ..oracle::GRAFT_SURGEON
};
