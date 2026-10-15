use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever a creature you control attacks" (CR 508.3a): once for each declared attacker, about that attacker. A
/// creature put onto the battlefield attacking was never declared (CR 508.4) and triggers nothing.
fn creature_you_control_attacks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.controller(*attacker) == q.controller(source) => Fired::about(Target::Object(*attacker)),
        _ => None,
    }
}

/// It gets +1/+0 until end of turn for each other attacking creature that shares a creature type with it. Counted as
/// the ability resolves (CR 608.2h), and the bonus is fixed for the turn (CR 611.2c).
fn bonus(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(attacker) = item.subject else { return Ok(()) };
    if !targets::permanent(g, attacker) {
        return Ok(());
    }
    let mine = g.chars(attacker).subtypes;
    let others: Vec<ObjectId> = g.state().attackers.iter().map(|a| a.id).filter(|&id| id != attacker).collect();
    let shares = |g: &dyn Game, id: ObjectId| {
        let c = g.chars(id);
        c.is(CardType::Creature) && Subtype::CREATURE_TYPES.iter().any(|&t| mine.has(t) && c.subtypes.has(t))
    };
    let count = others.into_iter().filter(|&id| shares(&*g, id)).count() as i16;
    if count > 0 {
        pump(g, item.object, attacker, count, 0);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature you control attacks, it gets +1/+0 until end of turn for each other attacking creature that shares a creature type with it.",
        listens: EventMask::ATTACKS,
        matches: creature_you_control_attacks,
        effect: bonus,
        ..TriggeredDef::BLANK
    })],
    ..oracle::SHARED_ANIMOSITY
};
