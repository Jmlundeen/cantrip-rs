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
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you attack with two or more creatures, draw a card.",
        listens: EventMask::ATTACKERS_DECLARED,
        matches: attack_with_two,
        effect: effects::draw::<1>,
        ..TriggeredDef::BLANK
    })],
    ..oracle::MILITARY_INTELLIGENCE
};
