use cantrip_api::*;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

/// "Exile X target artifacts and/or creatures. Return the exiled cards to the battlefield under their owners' control at the
/// beginning of the next end step." X is announced with the spell and counts the targets (CR 601.2b-c); those still legal as
/// it resolves are exiled together (CR 608.2b, 406.2, 101.4). The delayed triggered ability (CR 603.7) is about the cards
/// that are in exile now: a token exiled has ceased to exist (CR 111.7), a card an effect kept from being exiled is not one.
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled: Vec<ObjectId> = g.exile_all(&targets::legal_objects(g, item)).into_iter().filter(|&card| g.obj(card).zone.is(ZoneKind::Exile)).collect();
    if !exiled.is_empty() {
        delayed::at_next_end_step_about(g, item, 0, &exiled);
    }
    Ok(())
}

/// "Return the exiled cards to the battlefield under their owners' control": the cards the delayed ability carries, together
/// (CR 110.2a, 101.4); one that is no longer the card it was in exile is a new object and is not put there (CR 603.7c, 400.7), as
/// the move itself sees. A double-faced card returns with its front face up (the ruling).
fn return_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let back: Vec<(ObjectId, PlayerId)> = g.event_objects(item).iter().map(|&card| (card, g.obj(card).owner)).collect();
    g.put_all_onto_battlefield(&back)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE.counted()], exile),
    delayed: &[delayed_trigger(return_them, "Return the exiled cards to the battlefield under their owners' control.")],
    ..oracle::HIDE_ON_THE_CEILING
};
