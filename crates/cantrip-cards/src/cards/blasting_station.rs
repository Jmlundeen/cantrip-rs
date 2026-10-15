use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, effects};
use cantrip_oracle as oracle;

fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

/// "Whenever a creature enters": a creature under any player's control, and Blasting Station itself whenever it enters
/// as a creature (CR 603.6a).
fn creature_enters(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield) && q.chars(*new).is(CardType::Creature) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "You may untap this artifact", asked as the ability resolves (CR 608.2d).
fn may_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        g.untap(item.object);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice a creature: Blasting Station deals 1 damage to any target.",
            cost: Cost { tap: true, permanents: Some(PermanentCost { count: 1, filter: targets::filters::creature, to_hand: false }), ..Cost::NONE },
            targets: &[targets::ANY],
            effect: ping,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever a creature enters, you may untap Blasting Station.",
            listens: EventMask::ZONE_CHANGE,
            matches: creature_enters,
            effect: may_untap,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::BLASTING_STATION
};
