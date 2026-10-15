use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a creature you control or a creature spell you control becomes the target of a spell or ability an
/// opponent controls" (CR 603.2, 115.10): each thing named is its own becoming, so a spell naming two of your creatures
/// triggers it twice. An ability on the stack has no types, so it is never a creature spell.
fn your_creature_targeted(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameTarget { target: Target::Object(t), controller, .. }
            if *controller != q.controller(source)
                && q.controller(*t) == q.controller(source)
                && (q.is_on_battlefield(*t) || q.obj(*t).zone.is(ZoneKind::Stack))
                && q.chars(*t).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature you control or a creature spell you control becomes the target of a spell or ability an opponent controls, draw a card.",
        listens: EventMask::BECAME_TARGET,
        matches: your_creature_targeted,
        effect: effects::draw::<1>,
        ..TriggeredDef::BLANK
    })],
    ..oracle::SURRAK_ELUSIVE_HUNTER
};
