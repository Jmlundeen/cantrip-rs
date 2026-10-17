use cantrip_api::*;
use cantrip_oracle as oracle;

fn creature_leaves_without_dying(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, from, to, .. } = event else { return None };
    if *old != source && from.is(ZoneKind::Battlefield) && !to.is(ZoneKind::Graveyard)
        && q.controller(*old) == q.controller(source) && q.chars(*old).is(CardType::Creature)
    {
        Some(Fired { subject: Target::None, data: [0; 2] })
    } else {
        None
    }
}

fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

fn return_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = cantrip_mechanics::targets::object(g, item, 0) {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

fn another_creature_you_control(q: &dyn Query, asker: &Asker, target: Target) -> bool {
    matches!(target, Target::Object(id) if id != asker.source && cantrip_mechanics::targets::permanent(q, id)
        && q.controller(id) == asker.controller && q.chars(id).is(CardType::Creature))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, creature_leaves_without_dying), draw,
            "Whenever one or more other creatures you control leave the battlefield without dying, draw a card.")
            .once_per_batch().look_back(),
        ability(cost("{1}{U}").and(TAP), return_creature,
            "{1}{U}, {T}: Return another target creature you control to its owner's hand.")
            .targets(&[target(another_creature_you_control)]),
    ],
    ..oracle::DOUR_PORT_MAGE
};
