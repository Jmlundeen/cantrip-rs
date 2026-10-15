use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers};
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on the chosen creature; the second time this ability has resolved this turn, double the number
/// of +1/+1 counters on it instead (CR 701.10e, 608.2m). Earlier resolutions are this source's, counted in the turn's
/// history even when the Cub has left the battlefield.
fn landfall(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.targets[0] {
        if g.history().resolutions(item.object, item.ability) == 1 {
            double::counters(g, &[creature], CounterKind::PlusOne);
        } else {
            g.add_counters(creature, CounterKind::PlusOne, 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Landfall \u{2014} Whenever a land you control enters, put a +1/+1 counter on target creature you control. If this is the second time this ability has resolved this turn, double the number of +1/+1 counters on that creature instead.",
        listens: EventMask::ZONE_CHANGE,
        matches: triggers::landfall,
        targets: &[targets::CREATURE_YOU_CONTROL],
        effect: landfall,
        ..TriggeredDef::BLANK
    })],
    ..oracle::SCYTHECAT_CUB
};
