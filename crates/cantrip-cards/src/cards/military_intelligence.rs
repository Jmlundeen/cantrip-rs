use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever you attack with two or more creatures" (CR 508.1, 603.2): one trigger for the declaration, whose count is
/// the creatures declared together, so a creature put onto the battlefield attacking afterwards (CR 508.4) is not in it.
fn attack_with_two(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::AttackersDeclared { player, count } if *player == q.controller(source) && *count >= 2 => Fired::PLAIN,
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ATTACKERS_DECLARED, attack_with_two), |g, item| effects::draw(g, item, 1), "Whenever you attack with two or more creatures, draw a card.")],
    ..oracle::MILITARY_INTELLIGENCE
};
