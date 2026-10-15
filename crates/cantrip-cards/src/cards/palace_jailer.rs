use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// Exile target creature an opponent controls until an opponent becomes the monarch (CR 610.3). The wait outlives Palace Jailer:
/// it leaving returns nothing, and the game goes on watching. If an opponent became the monarch after this ability triggered
/// but before it resolved, the duration is over before the exile would begin and the creature does not move (CR 610.3b).
fn exile_until_crowned(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let entered = g
        .history()
        .all()
        .iter()
        .find(|r| matches!(r.event, Event::ZoneChange { new, to, .. } if new == item.object && to.is(ZoneKind::Battlefield)))
        .map(|r| r.serial);
    if let Some(since) = entered
        && g.history().all().iter().any(|r| r.serial > since && matches!(r.event, Event::BecameMonarch { player } if player != you))
    {
        return Ok(());
    }
    let Some(prey) = targets::object(g, item, 0) else { return Ok(()) };
    let exiled = g.exile_with(&[prey], item.object, false);
    if let Some(&card) = exiled.first() {
        let me = g.card_id(&CARD);
        g.delayed_trigger(DelayedWhen::UntilEvent, item.object, you, AbilityRef::delayed(me, 0), Target::Object(card));
    }
    Ok(())
}

/// "Until an opponent becomes the monarch": any opponent will do, and becoming it is the whole condition.
fn opponent_became_monarch(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameMonarch { player } if *player != q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// The exile is over: the card returns to the battlefield under its owner's control (nothing, once it is no longer that card).
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(card) = item.subject {
        let owner = g.obj(card).owner;
        g.put_onto_battlefield(card, owner, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, you become the monarch.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::become_monarch,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, exile target creature an opponent controls until an opponent becomes the monarch.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[cantrip_mechanics::targets::CREATURE_AN_OPPONENT_CONTROLS],
            effect: exile_until_crowned,
            ..TriggeredDef::BLANK
        }),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Until an opponent becomes the monarch, the exiled card stays exiled.",
        listens: EventMask::BECAME_MONARCH,
        matches: opponent_became_monarch,
        effect: return_it,
        ..TriggeredDef::BLANK
    })],
    ..oracle::PALACE_JAILER
};
