use cantrip_api::*;
use cantrip_mechanics::{choose, delayed, top_of_library};
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
    // "It" is the permanent: a card a replacement sent elsewhere (Containment Priest's exile) never became one, so it gains
    // nothing and nothing returns it (CR 603.7c, 400.7).
    if !g.is_on_battlefield(creature) {
        return Ok(());
    }
    cantrip_mechanics::effects::gain_haste(g, item.object, creature);
    delayed::at_next_end_step(g, item, 0, Target::Object(creature));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    delayed: &[delayed_trigger(delayed::return_subject_to_hand, "Return it to your hand.")],
    ..oracle::ANZRAGS_RAMPAGE
};
