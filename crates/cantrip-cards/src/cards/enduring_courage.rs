use cantrip_api::*;
use cantrip_mechanics::{enduring, haste_until_end_of_turn, pump, triggers};
use cantrip_oracle as oracle;

/// "It gets +2/+0 and gains haste until end of turn." Both are continuous effects of this ability (CR 611.2); the entering
/// creature is the trigger's subject, and if it has left the battlefield by now it is a new object (CR 400.7), so nothing happens.
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        instruction(g, |g| {
            pump(g, item.object, creature, 2, 0);
            haste_until_end_of_turn(g, item.object, creature);
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS,
            rally,
            "Whenever another creature you control enters, it gets +2/+0 and gains haste until end of turn.",
        ),
        triggered(when(EventMask::ZONE_CHANGE, enduring::dies_as_a_creature), enduring::return_as_enchantment, "When Enduring Courage dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)")
            .look_back(),
    ],
    ..oracle::ENDURING_COURAGE
};
