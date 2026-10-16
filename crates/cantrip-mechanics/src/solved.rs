//! Solved (CR 702.169) and the "to solve" ability of Case cards (CR 719.3). Case of the Crimson Pulse prints both.
//!
//! "Solved" is a designation a permanent can have (CR 719.3b): it has no rules meaning but to be a marker that spells and
//! abilities identify, it is neither an ability nor part of copiable values, and a permanent keeps it until it leaves the
//! battlefield. It is kept here as the Case's own indefinite effect granting it a marker ability that does nothing (the
//! Case card's first `delayed` ability, `MARKER`): an effect ends with the object (CR 400.7), is not copied (CR 707.2), and is
//! drawn on the card as a granted line, which is how a player sees that the Case is solved.
//!
//! A Case lists `MARKER` as its first `delayed` ability, its "To solve" as `triggered(solved::TO_SOLVE,
//! solved::become_solved, text).condition(..)` with the condition its own, and a "Solved" triggered ability with a matcher
//! that asks `is_solved` of the Case ("This ability triggers only if this Case is solved", CR 702.169c).

use cantrip_api::*;

use crate::triggers;

/// The marker a solved Case has been granted: it does nothing (CR 719.3b: "no rules meaning other than to act as a marker").
fn marked(_: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// The Case card's first `delayed` ability: the solved designation, granted to the permanent when it becomes solved.
pub const MARKER: AbilityDef = delayed_trigger(marked, "This Case is solved.");

/// The marker as the permanent's own card has it (a copy of a Case has the copied card's abilities, and was never solved).
fn marker_of(q: &dyn Query, id: ObjectId) -> AbilityRef {
    let o = q.state().obj(id);
    AbilityRef::delayed(o.copy.map_or(o.card, |c| c.card), 0)
}

/// Whether the permanent has the solved designation (CR 719.3b).
pub fn is_solved(q: &dyn Query, id: ObjectId) -> bool {
    let marker = marker_of(q, id);
    q.state().effects.iter().any(|e| e.affected == id && e.kind == EffectKind::AddAbility(marker))
}

/// "At the beginning of your end step, if this Case is not solved" (CR 719.3a): the event half of "To solve", for the Case's
/// controller; the Case's own condition is an intervening "if" beside it.
fn end_step_while_unsolved(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if is_solved(q, me.source) { None } else { triggers::your_end_step(q, me, event) }
}

/// The trigger of "To solve — [Condition]" (CR 719.3a).
pub const TO_SOLVE: Trigger = when_asked(EventMask::STEP_BEGAN, end_step_while_unsolved);

/// "This Case becomes solved" (CR 719.3a), when its "To solve" ability resolves with its condition (an intervening "if", CR
/// 603.4) still true. A Case that has left the battlefield is a new object and becomes nothing; one already solved stays so.
pub fn become_solved(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let case = item.object;
    if is_live(g, case) && g.is_on_battlefield(case) && !is_solved(g, case) {
        let marker = marker_of(g, case);
        g.add_effect(case, case, EffectKind::AddAbility(marker), Duration::Indefinite);
    }
    Ok(())
}
