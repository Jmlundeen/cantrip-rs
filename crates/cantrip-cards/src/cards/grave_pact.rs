use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Whenever a creature you control dies" (CR 700.4): judged on the board as it was before it died (CR 603.10a).
fn your_creature_dies(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.controller(*old) == q.controller(source)
                && q.chars(*old).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Every player but the controller sacrifices a creature of their choice (CR 701.21a): each chooses in turn order from
/// the active player, a lone creature is taken without asking, and every chosen creature is sacrificed at once.
fn each_other_sacrifices(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut doomed = Vec::new();
    for who in g.state().apnap().collect::<Vec<_>>() {
        if who == item.controller {
            continue;
        }
        let eligible: Vec<ObjectId> = g.permanents_of(who).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
        if eligible.len() <= 1 {
            doomed.extend(eligible);
            continue;
        }
        doomed.extend(choose::card(g, who, item.object, Purpose::Sacrifice, eligible)?);
    }
    g.sacrifice(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature you control dies, each other player sacrifices a creature of their choice.",
        listens: EventMask::ZONE_CHANGE,
        matches: your_creature_dies,
        effect: each_other_sacrifices,
        look_back: true,
        ..TriggeredDef::BLANK
    })],
    ..oracle::GRAVE_PACT
};
