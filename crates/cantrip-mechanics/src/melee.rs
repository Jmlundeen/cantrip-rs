//! Melee (CR 702.121a): "Whenever this creature attacks, it gets +1/+1 until end of turn for each opponent you attacked with a
//! creature this combat." Each instance triggers separately (CR 702.121b). Wulfgar of Icewind Dale prints it.
//!
//! A card with melee lists `TRIGGER` among its `abilities`.

use crate::pump;
use cantrip_api::*;

fn this_attacks(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => Fired::PLAIN,
        _ => None,
    }
}

/// The opponents `you` attacked with a creature this combat, counted as the ability resolves (the rulings): each player a
/// creature was declared attacking during `you`'s turn since this combat phase began, however many creatures, whether those
/// creatures are still attacking or the player still in the game; a planeswalker or battle attacked is not its controller,
/// and a creature put onto the battlefield attacking was never declared (CR 508.4).
fn opponents_attacked_this_combat(q: &dyn Query, you: PlayerId) -> i16 {
    let mut attacked: Vec<PlayerId> = Vec::new();
    for r in crate::history::this_combat(q) {
        if let Event::Attacks { defender: Target::Player(p), .. } = r.event
            && r.active == you
            && p != you
            && !attacked.contains(&p)
        {
            attacked.push(p);
        }
    }
    attacked.len() as i16
}

fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let n = opponents_attacked_this_combat(g, item.controller);
    pump(g, item.object, item.object, n, n);
    Ok(())
}

pub const TRIGGER: AbilityDef = triggered(
    when(EventMask::ATTACKS, this_attacks),
    grow,
    "Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for each opponent you attacked this combat.)",
);
