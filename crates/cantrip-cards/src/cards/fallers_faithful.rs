use cantrip_api::*;
use cantrip_mechanics::{history, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, destroy up to one other target creature. If that creature wasn't dealt damage this turn, its
/// controller draws two cards." With no target, or one that is illegal on resolution, none of it happens: "its controller" is
/// undefined (CR 608.2b). Whether it was dealt damage is read as it was before it is destroyed (CR 608.2h), damage that was
/// prevented is none dealt, and none is in the history (CR 615.6); its controller is the one it had then. A creature that survives (indestructible,
/// regeneration) still has its controller draw, the draw not depending on the destruction.
fn destroy_and_draw_two_unless_dealt_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(creature);
    let was_dealt_damage = history::was_dealt_damage_this_turn(g.history(), creature);
    g.destroy(creature);
    if !was_dealt_damage {
        g.draw(controller, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        destroy_and_draw_two_unless_dealt_damage,
        "When this creature enters, destroy up to one other target creature. If that creature wasn't dealt damage this turn, its controller draws two cards.",
    )
    .targets(&[up_to_one(targets::other_creature_filter)])],
    ..oracle::FALLERS_FAITHFUL
};
