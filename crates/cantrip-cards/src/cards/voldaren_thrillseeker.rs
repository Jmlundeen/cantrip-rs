use cantrip_api::*;
use cantrip_mechanics::{backup, damage_target, targets};
use cantrip_oracle as oracle;

/// Backup 2 (CR 702.165a): put two +1/+1 counters on target creature; if that's another creature, it gains the sacrifice
/// ability, fixed when backup triggers (CR 702.165d).
fn backup(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    backup::resolve(g, item, 2, &[], Some(0))
}

/// "{1}, Sacrifice this creature: It deals damage equal to its power to any target." The sacrifice is a cost (CR 601.2h), so
/// the creature is gone when this resolves and deals damage equal to its power as it last existed on the battlefield (CR 113.7a,
/// 608.2h); it is the source of the damage. A target no longer legal is dealt nothing (CR 608.2b).
fn blast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u16;
    damage_target(g, item, 0, power);
    Ok(())
}

const BLAST: AbilityDef =
    ability(cost("{1}").and(SACRIFICE_THIS), blast, "{1}, Sacrifice this creature: It deals damage equal to its power to any target.")
        .targets(&[targets::ANY]);

pub static CARD: CardDef = CardDef {
    abilities: &[
        backup::trigger("When this creature enters, put two +1/+1 counters on target creature. If that's another creature, it gains the following ability until end of turn.", backup),
        BLAST,
    ],
    delayed: &[BLAST],
    ..oracle::VOLDAREN_THRILLSEEKER
};
