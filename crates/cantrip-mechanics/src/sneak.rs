//! Sneak [cost] (CR 702.190a): "Any time you could cast an instant during your declare blockers step, you may cast this
//! spell by paying [cost] and returning an unblocked creature you control to its owner's hand rather than paying this
//! spell's mana cost." Shredder's Technique prints it.
//!
//! A card with it lists `way(cost, text)` among its `ways`: the alternative cost is the mana and the return of an unblocked
//! attacker (a cost chosen with the other costs, CR 601.2b, h), offered from the hand in the declare blockers step of its
//! controller's turn, at instant speed whatever the card's type (`WayDef::instant_speed`).
//!
//! CR 702.190b (a permanent spell cast for its sneak cost enters tapped and attacking what the returned creature attacked)
//! is not built: no card that is a permanent has sneak yet.

use cantrip_api::*;

use crate::combat::unblocked_attacker_you_control;

/// "During your declare blockers step" (CR 702.190a): the declare blockers step of a turn of the player casting it.
fn your_declare_blockers_step(q: &dyn Query, asker: &Asker) -> bool {
    let turn = &q.state().turn;
    turn.active == asker.controller && turn.step() == Step::DeclareBlockers
}

/// The alternative cost of sneak: `cost` and an unblocked attacker you control returned to its owner's hand. `text` is the
/// way's text, which a scenario's `Ability:` matches.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    WayDef {
        text,
        cost: Cost {
            mana: cost,
            permanents: Some(PermanentCost { count: 1, filter: unblocked_attacker_you_control, to_hand: true }),
            ..Cost::NONE
        },
        from: ZoneKind::Hand,
        exile_after: false,
        condition: Some(your_declare_blockers_step),
        instant_speed: true,
    }
}
