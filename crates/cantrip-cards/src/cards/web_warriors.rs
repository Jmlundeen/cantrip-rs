use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::plus_one_counter_on_each_other_creature_you_control,
        "When this creature enters, put a +1/+1 counter on each other creature you control.",
    )],
    ..oracle::WEB_WARRIORS
};
