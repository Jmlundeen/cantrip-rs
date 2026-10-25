use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on target creature you control, if it is still a legal target.
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::plus_one_counter_on_target(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT, counter, "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::LUMINARCH_ASPIRANT
};
