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

/// Sets up the card's delayed ability number 0 to trigger "this turn" (CR 603.7b): each time its `listens`/`matches`
/// accept an event, until the turn ends, controlled by the resolving ability's controller. `source` is the object the
/// ability's `matches` is asked of: the spell for "whenever a creature you control dies this turn", the creature it
/// is about for "when it dies this turn". A `matches` that says "you" is written `when_asked`: the ability's controller it
/// is told is this resolving ability's controller (CR 603.7d-e), whoever controls `source` later. Forth Eorlingas!, Glimpse of Nature, Rite of Harmony, Showdown of the Skalds,
/// Thunder of Unity, Warhost's Frenzy, Okoye and Turn Inside Out use it.
pub fn this_turn(g: &mut dyn Game, item: &StackItem, source: ObjectId) {
    let this = g.obj(item.object).card;
    g.delayed_trigger(DelayedWhen::ThisTurn, source, item.controller, AbilityRef::delayed(this, 0), Target::None);
}

/// Sets up the card's delayed ability number `ability` to trigger at the beginning of the next end step (CR 603.7), carrying
/// the objects `created` (CR 603.7c) to the moment it triggers, where `event_objects` of its stack item gives them back.
/// The ability is the card as the source has it now, or the card it copies. Mobilize's Warriors and Dalkovan Encampment's
/// use it with `sacrifice_created`.
pub fn at_next_end_step_about(g: &mut dyn Game, item: &StackItem, ability: u8, created: &[ObjectId]) {
    let o = *g.obj(item.object);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    g.delayed_trigger_about(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        item.object,
        item.controller,
        AbilityRef::delayed(card, ability),
        created,
    );
}

/// "Sacrifice them": the objects the delayed ability carries (`at_next_end_step_about`) that are still on the battlefield
/// as the objects they were (CR 603.7c, 400.7), sacrificed in one action; one another player controls now cannot be
/// sacrificed by this ability's controller (CR 701.21a).
pub fn sacrifice_created(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let survivors: Vec<ObjectId> = g
        .event_objects(item)
        .iter()
        .copied()
        .filter(|&id| is_live(g, id) && g.is_on_battlefield(id) && g.controller(id) == item.controller)
        .collect();
    g.sacrifice(&survivors);
    Ok(())
}

/// The delayed ability of "sacrifice them at the beginning of the next end step" (CR 603.7c), the body of
/// `sacrifice_created`; a card lists it among its `delayed` abilities and names its index in `at_next_end_step_about`.
pub const SACRIFICE_CREATED: AbilityDef = delayed_trigger(sacrifice_created, "Sacrifice them at the beginning of the next end step.");

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

