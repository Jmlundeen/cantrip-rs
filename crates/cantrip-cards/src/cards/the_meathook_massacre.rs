use cantrip_api::*;
use cantrip_mechanics::{effects, pump};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "Each creature gets -X/-X until end of turn": every creature on the battlefield as this resolves, fixed then
/// (CR 611.2c), for the X it was cast with (entering without being cast, X is 0).
fn shrink_all(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.obj(item.object).x as i16;
    if x == 0 {
        return Ok(());
    }
    let creatures: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for creature in creatures {
        pump(g, item.object, creature, -x, -x);
    }
    Ok(())
}

/// "Whenever a creature an opponent controls dies" (CR 700.4): judged on the board as it was before it died
/// (CR 603.10a), by who controlled it there.
fn opponents_dies(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.chars(*old).is(CardType::Creature)
                && q.controller(*old) != q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Each opponent loses 1 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When The Meathook Massacre enters, each creature gets -X/-X until end of turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: shrink_all,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever a creature you control dies, each opponent loses 1 life.",
            listens: EventMask::ZONE_CHANGE,
            matches: cantrip_mechanics::triggers::creature_you_control_dies,
            effect: drain,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever a creature an opponent controls dies, you gain 1 life.",
            listens: EventMask::ZONE_CHANGE,
            matches: opponents_dies,
            effect: effects::gain_life::<1>,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::THE_MEATHOOK_MASSACRE
};
