//! "You may cast this spell as though it had flash. If you cast it any time a sorcery couldn't have been cast, the controller
//! of the permanent it becomes sacrifices it at the beginning of the next cleanup step." Weatherlight's cycle prints it:
//! Necromancy, Armor of Thorns, Grave Servitude, Lightning Reflexes, Mystic Veil, Parapet, Relic Ward, Soar, Spider Climb
//! and Ward of Lights.
//!
//! A card with it sets `cast_as_though_flash`, lists `SACRIFICE` first among its `delayed` abilities and `RIDER` as its
//! `untimely_cleanup_rider`: the second sentence is a static ability that functions on the stack, so the core creates the
//! delayed triggered ability as the spell becomes a permanent (CR 608.3g, 603.7a), about that permanent, if the spell was
//! cast at a time a sorcery couldn't have been by its own permission. It triggers at the beginning of the next cleanup
//! step and goes on the stack after the hand-size discard and damage wearing off, and players get priority (CR 514.3a).

use cantrip_api::*;

/// The rider names the card's first delayed ability, `SACRIFICE`.
pub const RIDER: Option<u8> = Some(0);

/// "The controller of the permanent it becomes sacrifices it": the permanent the delayed ability is about, if it is still
/// that object on the battlefield (CR 603.7c, 400.7), sacrificed by whoever controls it as the ability resolves (CR
/// 701.21a), not by the ability's controller.
fn sacrifice_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(it) = item.subject
        && is_live(g, it)
        && g.is_on_battlefield(it)
    {
        let controller = g.controller(it);
        g.sacrifice(controller, &[it]);
    }
    Ok(())
}

/// The delayed triggered ability the rider creates.
pub const SACRIFICE: AbilityDef =
    delayed_trigger(sacrifice_it, "At the beginning of the next cleanup step, the controller of the permanent it became sacrifices it.");
