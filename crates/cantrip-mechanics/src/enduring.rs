//! The Enduring creatures' last ability: "When this dies, if it was a creature, return it to the battlefield under its
//! owner's control. It's an enchantment. (It's not a creature.)" Enduring Innocence, Enduring Curiosity and Enduring Courage print it.

use cantrip_api::*;

/// "When this dies, if it was a creature" (CR 700.4, 603.4): judged on what it last was on the battlefield (CR 603.10a).
/// Once it comes back an enchantment, it is not a creature when it dies again and the ability does not trigger.
pub fn dies_as_a_creature(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.chars(source).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)": a layer-4
/// effect with no end (CR 611.2a, 613.1d), for as long as this is the object that returned.
pub fn return_as_enchantment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    let owner = g.obj(card).owner;
    let returned = g.put_onto_battlefield(card, owner, false)?;
    if g.is_on_battlefield(returned) {
        g.add_effect(item.object, returned, EffectKind::SetTypes(TypeSet::of(&[CardType::Enchantment])), Duration::Indefinite);
    }
    Ok(())
}
