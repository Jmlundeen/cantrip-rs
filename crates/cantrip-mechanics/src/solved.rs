//! Cases (CR 719): "To solve" and "Solved" (CR 702.169), the two keyword abilities every Case prints, and the solved
//! designation they turn on. Case of the Crimson Pulse, Case of the Locked Hothouse, Case of the Stashed Skeleton and Case of
//! the Uneaten Feast use it; a Case's file says only its condition and its solved ability.
//!
//! Solved is a designation (CR 719.3b), the core's `Marks::solved`: no ability, no copiable value, kept until the permanent
//! leaves the battlefield. So a permanent that becomes a copy of a solved Case is not solved, a solved Case that becomes a
//! copy of another Case stays solved, a Case that leaves and comes back is a new, unsolved object (CR 400.7), and a Case that
//! loses all its abilities stays solved though its "Solved" ability is gone.
//!
//! - "To solve - [condition]" means "At the beginning of your end step, if [condition] and this Case is not solved, this Case
//!   becomes solved" (CR 719.3a): `to_solve(condition, text)`. Both halves are an intervening "if" (CR 603.4), asked as the end
//!   step begins and again as the ability resolves (the rulings).
//! - "Solved - [triggered ability]": "This ability triggers only if this Case is solved" (CR 702.169c), asked with the
//!   trigger event and not again: `triggered(trigger, effect, text)`.
//! - "Solved - [activated ability]": "Activate only if this Case is solved" (CR 702.169d): `ability(cost, effect, text)`.
//! - "Solved - [static ability]": "As long as this Case is solved, [static ability]" (CR 702.169b): a play rule is
//!   `play_rules(&[..])`; a static ability of the layers asks `solved_in_layers` of its source.

use cantrip_api::*;

use crate::triggers;

/// Whether the permanent has the solved designation (CR 719.3b).
pub fn is_solved(q: &dyn Query, id: ObjectId) -> bool {
    q.is_solved(id)
}

/// "This Case is solved", of the ability's source.
fn this_case_is_solved(q: &dyn Query, asker: &Asker) -> bool {
    q.is_solved(asker.source)
}

/// Whether a static ability's source is solved, read from the layers (CR 702.169b, 719.3c): "As long as this Case is solved".
pub fn solved_in_layers(view: &dyn LayerQuery, source: ObjectId) -> bool {
    view.state().obj(source).marks.solved
}

/// "At the beginning of your end step, if ... this Case is not solved" (CR 719.3a): the event half of "To solve", for the
/// Case's controller, with the "not solved" half of its intervening "if" asked as it triggers. Asked again as it resolves by
/// `become_solved`, which does nothing to a Case that is solved by then.
fn your_end_step_while_unsolved(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if q.is_solved(me.source) { None } else { triggers::your_end_step(q, me, event) }
}

/// "This Case becomes solved" (CR 719.3a), as its "To solve" ability resolves with its condition still true (CR 603.4). A
/// Case that has left the battlefield is a new object and becomes nothing, and one already solved stays as it is (the
/// core's `solve`).
fn become_solved(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.solve(item.object);
    Ok(())
}

/// "To solve - [condition]" (CR 719.3a): at the beginning of its controller's end step, if `condition` holds for the Case
/// and it is not solved, it becomes solved. `condition` is the Case's own, an intervening "if" asked as the step begins and
/// as the ability resolves.
pub const fn to_solve(condition: Condition, text: &'static str) -> AbilityDef {
    triggered_ability(when_asked(EventMask::STEP_BEGAN, your_end_step_while_unsolved), become_solved, text).condition(condition)
}

const fn triggered_ability(trigger: Trigger, effect: Effect, text: &'static str) -> AbilityDef {
    cantrip_api::triggered(trigger, effect, text)
}

/// "Solved - [triggered ability]" (CR 702.169c): the ability, which triggers only if this Case is solved.
pub const fn triggered(trigger: Trigger, effect: Effect, text: &'static str) -> AbilityDef {
    triggered_ability(trigger, effect, text).only_if(this_case_is_solved)
}

/// "Solved - [activated ability]" (CR 702.169d): the ability, activated only if this Case is solved.
pub const fn ability(cost: Cost, effect: Effect, text: &'static str) -> AbilityDef {
    cantrip_api::ability(cost, effect, text).condition(this_case_is_solved)
}

/// "Solved - [static ability]" of play rules (CR 702.169b): as long as this Case is solved, `rules`.
pub const fn play_rules(rules: &'static [PlayRule]) -> PlayRule {
    PlayRule::While(this_case_is_solved, rules)
}
