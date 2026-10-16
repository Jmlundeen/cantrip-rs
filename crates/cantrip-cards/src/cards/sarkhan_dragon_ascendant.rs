use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{behold, tokens};
use cantrip_oracle as oracle;

/// "When Sarkhan enters, you may behold a Dragon. If you do, create a Treasure token." (CR 701.4a, 608.2d): the choice of a
/// Dragon permanent you control or a Dragon card revealed from your hand is made as the ability resolves, and the Treasure
/// follows only if one was beheld (nothing is asked, and no Treasure made, with no Dragon to behold).
fn behold_a_dragon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if behold::on_resolution(g, item.controller, item.object, behold::dragon, "Dragon")? {
        tokens::create_treasure(g, item.controller)?;
    }
    Ok(())
}

/// "Whenever a Dragon you control enters" (CR 603.6a): any permanent that is a Dragon as it enters, under your control (the
/// word read through Sarkhan's own text, CR 612).
fn dragon_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).has_subtype(q.words(me.source).subtype(Subtype::Dragon)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Put a +1/+1 counter on Sarkhan. Until end of turn, Sarkhan becomes a Dragon in addition to his other types and gains
/// flying." (CR 611.2, 205.1b, 613.1): he gets the counter, the Dragon type (a layer 4
/// effect that adds to his types, read through his text's creature type word) and flying (layer 6) until the turn ends.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let sarkhan = item.object;
    g.add_counters(sarkhan, CounterKind::PlusOne, 1);
    let dragon = g.words(sarkhan).subtype(Subtype::Dragon);
    g.add_effect(sarkhan, sarkhan, EffectKind::AddSubtype(dragon), Duration::EndOfTurn);
    g.add_effect(sarkhan, sarkhan, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, behold_a_dragon, "When Sarkhan enters, you may behold a Dragon. If you do, create a Treasure token."),
        triggered(when_asked(EventMask::ZONE_CHANGE, dragon_you_control_enters), grow, "Whenever a Dragon you control enters, put a +1/+1 counter on Sarkhan. Until end of turn, Sarkhan becomes a Dragon in addition to his other types and gains flying."),
    ],
    ..oracle::SARKHAN_DRAGON_ASCENDANT
};
