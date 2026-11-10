//! The Clockwork recipe: "This creature enters with N +1/+0 counters on it. At end of combat, if this creature attacked or
//! blocked this combat, remove a +1/+0 counter from it. {X}, {T}: Put up to X +1/+0 counters on this creature. This ability
//! can't cause the total number of +1/+0 counters on this creature to be greater than N. Activate only during your upkeep."
//! Clockwork Avian and Clockwork Beast print it (N is 4 and 7).
//!
//! A card lists `clockwork::end_of_combat(text)` and `clockwork::recharge::<N>(text)` among its abilities and sets
//! `enters: Some(clockwork::enters::<N>)`.

use crate::{choose, history, triggers};
use cantrip_api::*;

/// "This creature enters with N +1/+0 counters on it" (CR 614.1c, 122.6). Clockwork Avian and Clockwork Beast print it.
pub fn enters<const N: u8>(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Plus1Plus0 as usize] += i32::from(N);
    Ok(())
}

/// "if this creature attacked or blocked this combat" (CR 603.4): it was declared as an attacker or as a blocker this combat.
fn attacked_or_blocked(q: &dyn Query, asker: &Asker) -> bool {
    history::this_combat(q).iter().any(|r: &Record| match &r.event {
        Event::Attacks { attacker, .. } => *attacker == asker.source,
        Event::Blocks { blocker, declared: true, .. } => *blocker == asker.source,
        _ => false,
    })
}

fn remove_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        let _ = g.remove_counters(item.object, CounterKind::Plus1Plus0, 1);
    });
    Ok(())
}

/// "At end of combat, if this creature attacked or blocked this combat, remove a +1/+0 counter from it." (CR 603.4, 122.1).
/// Clockwork Avian and Clockwork Beast print it.
pub const fn end_of_combat(text: &'static str) -> AbilityDef {
    triggered(triggers::END_OF_COMBAT, remove_counter, text).condition(attacked_or_blocked)
}

fn put_counters<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let chosen = choose::number(g, item.controller, item.object, 0, u32::from(item.x), "Put counters")?;
    g.add_counters_with_limit(item.object, CounterKind::Plus1Plus0, chosen as i32, i32::from(N));
    Ok(())
}

/// "{X}, {T}: Put up to X +1/+0 counters on this creature. This ability can't cause the total number of +1/+0 counters on this
/// creature to be greater than N. Activate only during your upkeep." (CR 122.1, 602.5b). Clockwork Avian and Clockwork Beast
/// print it.
pub const fn recharge<const N: u8>(text: &'static str) -> AbilityDef {
    ability(cost("{X}").and(TAP), put_counters::<N>, text).condition(crate::your_upkeep)
}
