use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, effects};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Untap", "untap it?", "", "Untap", "Leave it tapped");

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
    if effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        g.untap(item.object);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP.and(sacrifice(1, targets::filters::creature)), ping, "{T}, Sacrifice a creature: Blasting Station deals 1 damage to any target.")
            .targets(&[targets::ANY]),
        triggered(when(EventMask::ZONE_CHANGE, creature_enters), may_untap, "Whenever a creature enters, you may untap Blasting Station."),
    ],
    ..oracle::BLASTING_STATION
};
