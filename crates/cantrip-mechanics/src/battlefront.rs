//! Battlefront (Star Trek): an ability word whose text reads "..., where X is the number of opponents you attacked this
//! turn." X counts each opponent a creature of yours was declared attacking this turn, once each however many creatures
//! attacked them, and whether or not those creatures are still attacking, still on the battlefield or still in the game;
//! a creature put onto the battlefield attacking was never declared (CR 508.4) and a planeswalker or battle attacked is
//! not its controller (CR 508.5). Korok, Liberated Rage and Martok, Hero of the Empire print it.
//!
//! A card with battlefront reads X with `opponents_attacked_this_turn` and takes its trigger with `you_attack` (once, for
//! "Whenever you attack", CR 508.1) or with `triggered` on `triggers::THIS_ATTACKS` for "Whenever this creature attacks".

use cantrip_api::*;

/// The opponents `you` attacked with a creature this turn (CR 508.1): each player one of your creatures was declared
/// attacking during this turn, counted as it is read, whatever their number, their state now and whether that player is
/// still in the game. Battlefront's X.
pub fn opponents_attacked_this_turn(q: &dyn Query, you: PlayerId) -> u16 {
    let mut attacked: Vec<PlayerId> = Vec::new();
    for r in q.history().this_turn() {
        if let Event::Attacks { defender: Target::Player(p), .. } = r.event
            && r.active == you
            && p != you
            && !attacked.contains(&p)
        {
            attacked.push(p);
        }
    }
    attacked.len() as u16
}

/// "Whenever you attack" (CR 508.1, 603.2c): one declaration of attackers by the ability's controller, whatever the
/// number of attackers, is one trigger.
pub fn you_attack(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::AttackersDeclared { .. } if q.state().turn.active == me.controller => Fired::PLAIN,
        _ => None,
    }
}
