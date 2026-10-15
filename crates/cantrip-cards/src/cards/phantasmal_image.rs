use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "When this creature becomes the target of a spell or ability, sacrifice it."
fn targeted(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameTarget { target: Target::Object(id), .. } if *id == source => Fired::PLAIN,
        _ => None,
    }
}

/// The ability the copy's exception adds (CR 707.9b). Not a card anyone plays: a definition for the exception's
/// ability to be referred to by.
pub static EXCEPTION: CardDef = CardDef {
    name: "Phantasmal Image (copy exception)",
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature becomes the target of a spell or ability, sacrifice it.",
        listens: EventMask::BECAME_TARGET,
        matches: targeted,
        effect: effects::sacrifice_self,
        ..TriggeredDef::BLANK
    })],
    ..CardDef::BLANK
};

/// "You may have this creature enter as a copy of any creature on the battlefield, except it's an Illusion in
/// addition to its other types and it has [the exception's ability]." A choice, not a target (CR 608.2d): with one
/// creature out it is taken without asking.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |q, copy| {
        let exception = AbilityRef { card: q.card_id(&EXCEPTION), index: 0 };
        copy.with_ability(exception)
    })?;
    // "It's an Illusion": the creature type is a word of this card's text (CR 612), so it is the type that text names now.
    let illusion = g.words(entry.object).subtype(Subtype::Illusion);
    entry.copy = entry.copy.map(|copy| copy.with_subtype(illusion));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enter_as_copy),
    ..oracle::PHANTASMAL_IMAGE
};

pub static EXTRA: &[&CardDef] = &[&EXCEPTION];
