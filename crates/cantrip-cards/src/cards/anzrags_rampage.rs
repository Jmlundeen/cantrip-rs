use cantrip_api::*;
use cantrip_mechanics::{choose, delayed, targets, top_of_library};
use cantrip_oracle as oracle;

/// "The number of artifacts that were put into graveyards from the battlefield this turn" (CR 700.4): every move from the
/// battlefield to a graveyard this turn, of anybody's, of an object that was an artifact as it left (last known
/// information), for any reason, tokens that have since ceased to exist included.
fn artifacts_put_into_graveyards_this_turn(q: &dyn Query) -> usize {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Battlefield).to(ZoneKind::Graveyard).this_turn())
        .filter(|r| matches!(r.event, Event::ZoneChange { old, .. } if q.state().last_known(old).is_some_and(|l| l.types.has(CardType::Artifact))))
        .count()
}

/// "Destroy all artifacts you don't control, then exile the top X cards of your library, where X is the number of
/// artifacts that were put into graveyards from the battlefield this turn" (CR 701.8, 406.2): the artifacts go together
/// and are counted after they have gone. "You may put a creature card exiled this way onto the battlefield. It gains
/// haste. Return it to your hand at the beginning of the next end step": nothing is asked when no creature card was
/// exiled; the haste lasts as long as the object does and the delayed return names the object (CR 603.7c, 400.7).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let artifacts: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Artifact) && g.controller(id) != you).collect();
    g.destroy_all(&artifacts);
    let x = artifacts_put_into_graveyards_this_turn(g);
    let top = top_of_library(g, you, x);
    let exiled = g.exile_all(&top);
    let creatures: Vec<ObjectId> =
        exiled.iter().copied().filter(|&c| g.obj(c).zone.is(ZoneKind::Exile) && g.chars(c).is(CardType::Creature)).collect();
    let Some(&card) = choose::cards(g, you, item.object, Purpose::PutOntoBattlefield, creatures, 1, true)?.first() else { return Ok(()) };
    let creature = g.put_onto_battlefield(card, you, false)?;
    if targets::permanent(g, creature) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::Indefinite);
        delayed::at_next_end_step(g, item, 0, Target::Object(creature));
    }
    Ok(())
}

/// "Return it to your hand": the creature, if it is still the object that came onto the battlefield (CR 603.7c, 400.7) and
/// still there, whoever controls it now (CR 400.3: it goes to its owner's hand, which is the player whose library it
/// was exiled from).
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(it) = item.subject
        && targets::permanent(g, it)
    {
        g.return_to_hand(&[it]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    delayed: &[delayed_trigger(return_it, "Return it to your hand.")],
    ..oracle::ANZRAGS_RAMPAGE
};
