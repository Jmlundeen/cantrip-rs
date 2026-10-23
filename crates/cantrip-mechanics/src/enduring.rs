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

/// "It's an enchantment. (It's not a creature.)": what it returns as (`EntersAs`), from the moment it enters (CR 614.12).
static AN_ENCHANTMENT: EntersAs = EntersAs { types: Some(TypeSet::of(&[CardType::Enchantment])), ..EntersAs::NONE };

/// "Return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)": it enters as an
/// enchantment, never as a creature (CR 614.12), and stays one for as long as it is the object that returned (CR 611.2a,
/// 613.1d).
pub fn return_as_enchantment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield_as(card, owner, item, &AN_ENCHANTMENT)?;
    Ok(())
}
