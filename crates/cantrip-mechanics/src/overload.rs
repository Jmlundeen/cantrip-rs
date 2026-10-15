//! Overload [cost] (CR 702.96a): "You may cast this spell by paying [cost] rather than its mana cost. If you do, change
//! its text by replacing all instances of 'target' with 'each'." Damn, Vandalblast and Winds of Abandon print it.
//!
//! The cost is the card's `overload` field and the way it was cast is `WAY_OVERLOAD`, both of the core. What the spell
//! affects is `affected`: every permanent that fits when overloaded, the one legal target when not.

use cantrip_api::*;

/// The permanents the spell affects: when overloaded, every permanent on the battlefield for which `fits` holds, given
/// its characteristics and its controller; otherwise the spell's target, if it is still legal (CR 608.2b).
pub fn affected(g: &mut dyn Game, item: &StackItem, fits: impl Fn(&Chars, PlayerId) -> bool) -> Vec<ObjectId> {
    if item.way == WAY_OVERLOAD {
        let all: Vec<ObjectId> = g.state().battlefield().iter().copied().collect();
        all.into_iter().filter(|&id| fits(&g.chars(id), g.controller(id))).collect()
    } else if g.target_legal(item, 0)
        && let Target::Object(permanent) = item.targets[0]
    {
        vec![permanent]
    } else {
        Vec::new()
    }
}
