//! Raid (CR 207.2c, an ability word): "Raid \u{2014} When this creature enters, if you attacked this turn, [effect]." It has no
//! rules meaning of its own; "if you attacked this turn" is an intervening "if" (CR 603.4) about the declarations of attackers
//! the ability's controller made this turn (CR 508.1): one attacker is enough, whichever it was and whomever it attacked, and
//! it need not still be on the battlefield (the Raid rulings). Fire Nation Raider, Mardu Hordechief, Skyship Buccaneer and
//! Wingmate Roc print it.
//!
//! A card with a raid enters trigger lists `enters(effect, text)` among its `abilities`.

use cantrip_api::*;

use crate::history::creatures_attacked_with;
use crate::triggers::THIS_ENTERS;

/// "If you attacked this turn": checked as the ability triggers and again as it resolves (CR 603.4). Fire Nation Engineer asks it
/// at the beginning of the end step.
pub fn you_attacked_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    creatures_attacked_with(q, asker.controller) > 0
}

/// "Raid \u{2014} When this creature enters, if you attacked this turn, [effect]." `text` is the ability's full text, as printed.
pub const fn enters(effect: fn(&mut dyn Game, &StackItem) -> Result<(), Suspend>, text: &'static str) -> AbilityDef {
    triggered(THIS_ENTERS, effect, text).condition(you_attacked_this_turn)
}
