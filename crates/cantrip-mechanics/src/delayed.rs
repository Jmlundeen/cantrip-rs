//! "At the beginning of the next end step, [do something] to it" (CR 603.7): a delayed triggered ability a spell or ability
//! sets up about one object. The card lists the ability in its `delayed`; these register it and are the bodies it names.

use cantrip_api::*;

/// Sets up the card's delayed ability number `ability` (an index into its `delayed` list) to trigger at the beginning of
/// the next end step (CR 603.7), about `subject`, controlled by the resolving ability's controller. Sneak Attack, Through
/// the Breach, Flickerwisp and Daretti, Scrap Savant's emblem use it.
pub fn at_next_end_step(g: &mut dyn Game, item: &StackItem, ability: u8, subject: Target) {
    let this = g.obj(item.object).card;
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        item.object,
        item.controller,
        AbilityRef::delayed(this, ability),
        subject,
    );
}

/// "Sacrifice it": the delayed ability's subject, if it is still the object it was on the battlefield (CR 603.7c, 400.7)
/// and its controller is the one sacrificing it (CR 701.21a). Sneak Attack and Through the Breach print it at the next
/// end step.
pub fn sacrifice_subject(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(it) = item.subject
        && is_live(g, it)
        && g.is_on_battlefield(it)
        && g.controller(it) == item.controller
    {
        g.sacrifice(&[it]);
    }
    Ok(())
}

/// "Exile it": the delayed ability's subject, if it is still the object it was on the battlefield (CR 603.7c, 400.7).
/// Shallow Grave, Goryo's Vengeance and Corpse Dance print it at the next end step.
pub fn exile_subject(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(it) = item.subject
        && is_live(g, it)
        && g.is_on_battlefield(it)
    {
        g.exile(it);
    }
    Ok(())
}

/// "Return that card to the battlefield under its owner's control": the delayed ability's subject, if it is still the
/// card it was when the ability was set up (a new object if it left its zone, CR 603.7c, 400.7). Flickerwisp and Daretti,
/// Scrap Savant's emblem print it at the next end step.
pub fn return_subject_to_battlefield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    if !is_live(g, card) {
        return Ok(());
    }
    let owner = g.obj(card).owner;
    g.put_onto_battlefield(card, owner, false)?;
    Ok(())
}

