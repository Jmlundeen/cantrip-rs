use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a creature dies" (CR 700.4): any creature, anyone's, judged on the board as it was before it died
/// (CR 603.10a). The ability carries the player who controlled it as it last existed on the battlefield (CR 608.2h),
/// who is not necessarily Fecundity's controller.
fn creature_dies(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.chars(*old).is(CardType::Creature) =>
        {
            Fired::about(Target::Player(q.controller(*old)))
        }
        _ => None,
    }
}

/// That creature's controller may draw a card; a player who has left the game is asked nothing.
fn may_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(who) = item.subject else { return Ok(()) };
    if !g.state().in_game(who) {
        return Ok(());
    }
    if cantrip_mechanics::effects::ask_may(g, who, item.object, Purpose::Optional)? {
        g.draw(who, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature dies, that creature's controller may draw a card.",
        listens: EventMask::ZONE_CHANGE,
        matches: creature_dies,
        effect: may_draw,
        look_back: true,
        ..TriggeredDef::BLANK
    })],
    ..oracle::FECUNDITY
};
