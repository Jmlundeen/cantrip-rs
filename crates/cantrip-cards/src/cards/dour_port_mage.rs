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

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, creature_leaves_without_dying), draw,
            "Whenever one or more other creatures you control leave the battlefield without dying, draw a card.")
            .once_per_batch(),
        ability(cost("{1}{U}").and(TAP), return_creature,
            "{1}{U}, {T}: Return another target creature you control to its owner's hand.")
            .targets(&[cantrip_mechanics::targets::OTHER_CREATURE_YOU_CONTROL]),
    ],
    ..oracle::DOUR_PORT_MAGE
};
