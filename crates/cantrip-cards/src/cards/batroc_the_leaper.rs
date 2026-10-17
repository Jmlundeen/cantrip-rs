use cantrip_api::*;
use cantrip_mechanics::{kicker, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// He deals damage equal to his power to each of up to X targets, X being the number of times he was kicked: the
/// targets are chosen as the trigger goes on the stack, counted by the kicks (`TargetSpec::COUNTED`).
fn damage_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u16;
    cantrip_mechanics::damage_each_target(g, item, power);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: multikicker(mana("{2}")),
    enters: Some(kicker::enters_with_counters::<{ CounterKind::PlusOne as usize }>),
    abilities: &[triggered(THIS_ENTERS, damage_each, "When Batroc enters, he deals damage equal to his power to each of up to X targets, where X is the number of times he was kicked.")
        .targets(&[targets::ANY.between(0, TargetSpec::COUNTED)])],
    ..oracle::BATROC_THE_LEAPER
};
