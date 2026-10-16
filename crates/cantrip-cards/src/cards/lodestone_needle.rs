use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{craft, explore, targets};
use cantrip_oracle as oracle;

/// "Tap up to one target artifact or creature and put two stun counters on it" (CR 122.1d): one already tapped just gets
/// the counters (the card's ruling).
fn tap_and_stun(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.tap(permanent);
        g.add_counters(permanent, CounterKind::Stun, 2);
    }
    Ok(())
}

/// "Target creature you control explores." (CR 701.44) Its one target illegal, the ability does not resolve at all (CR
/// 608.2b), so the target is the creature.
fn target_explores(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.targets[0] else { return Ok(()) };
    explore(g, creature)
}

pub static BACK: CardDef = CardDef {
    abilities: &[ability(cost("{1}, {T}"), target_explores, "{1}, {T}: Target creature you control explores. Activate only as a sorcery.")
        .targets(&[targets::CREATURE_YOU_CONTROL])
        .sorcery_speed()],
    ..oracle::GUIDESTONE_COMPASS
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, tap_and_stun, "When this artifact enters, tap up to one target artifact or creature and put two stun counters on it.")
            .targets(&[up_to_one(targets::artifact_or_creature_filter)]),
        craft::ability(mana("{2}{U}"), &craft::ARTIFACT, "Craft with artifact {2}{U}"),
    ],
    back: Some(&BACK),
    ..oracle::LODESTONE_NEEDLE
};
