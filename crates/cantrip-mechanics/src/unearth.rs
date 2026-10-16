//! Unearth [cost] (CR 702.84a): "[Cost]: Return this card from your graveyard to the battlefield. It gains haste. Exile it
//! at the beginning of the next end step or if it would leave the battlefield. Unearth only as a sorcery." Phyrexian
//! Dragon Engine prints it.
//!
//! A card with unearth lists `ability(cost, text)` among its `abilities` and `EXILE` as its first `delayed` ability.

use cantrip_api::*;

/// Return the card from the graveyard, hasty, and exile it at the beginning of the next end step (the card's delayed
/// ability 0) or if it would leave the battlefield. Nothing if it has left the graveyard.
fn unearth(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    if g.obj(card).became.is_some() || !g.obj(card).zone.is(ZoneKind::Graveyard) {
        return Ok(());
    }
    let returned = g.put_onto_battlefield(card, item.controller, false)?;
    if !g.is_on_battlefield(returned) {
        return Ok(());
    }
    crate::effects::gain_haste(g, item.object, returned);
    g.exile_on_leave(returned);
    let this = g.obj(returned).card;
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        returned,
        item.controller,
        AbilityRef::delayed(this, 0),
        Target::Object(returned),
    );
    Ok(())
}

pub const fn ability(cost: ManaCost, text: &'static str) -> AbilityDef {
    cantrip_api::ability(Cost::mana(cost), unearth, text)
        .sorcery_speed()
        .from(ZoneKind::Graveyard)
}

fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(it) = item.subject
        && g.is_on_battlefield(it)
        && g.obj(it).became.is_none()
    {
        g.exile(it);
    }
    Ok(())
}

pub const EXILE: AbilityDef = delayed_trigger(exile_it, "Exile it.");
