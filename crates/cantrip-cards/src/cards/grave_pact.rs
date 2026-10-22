use cantrip_api::*;
use cantrip_mechanics::each_opponent_sacrifices;
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

/// Every player but the controller sacrifices a creature of their choice (CR 701.21a, 101.4): each chooses in turn order
/// from the active player, a lone creature is taken without asking, and every chosen creature is sacrificed at once.
fn each_other_sacrifices(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| q.chars(id).is(CardType::Creature))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, your_creature_dies), each_other_sacrifices, "Whenever a creature you control dies, each other player sacrifices a creature of their choice.")],
    ..oracle::GRAVE_PACT
};
