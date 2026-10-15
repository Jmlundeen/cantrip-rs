use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on target creature you control, if it is still a legal target.
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control.",
        listens: EventMask::STEP_BEGAN,
        matches: cantrip_mechanics::triggers::beginning_of_your_combat,
        targets: &[targets::CREATURE_YOU_CONTROL],
        effect: counter,
        ..TriggeredDef::BLANK
    })],
    ..oracle::LUMINARCH_ASPIRANT
};
