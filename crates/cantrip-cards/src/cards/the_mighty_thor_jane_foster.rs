use cantrip_api::*;
use cantrip_mechanics::attach::equipment_you_control_enters;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Up to one target nontoken artifact or creature" (CR 115.1, 111.1): a permanent that is a card and is an artifact or a creature.
fn nontoken_artifact_or_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::artifact_or_creature_filter(q, asker, t) && matches!(t, Target::Object(id) if !q.obj(id).token)
}

/// "Exile up to one target nontoken artifact or creature, then return that card to the battlefield tapped under its owner's
/// control": exiled, then returned as a new object (CR 400.7), entering tapped (CR 110.5b). The target is a card, never a token.
fn exile_and_return_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let returning: Vec<(ObjectId, PlayerId)> =
        g.exile_all(&[permanent]).into_iter().filter(|&card| g.obj(card).zone.is(ZoneKind::Exile)).map(|card| (card, g.obj(card).owner)).collect();
    for (card, owner) in returning {
        g.put_onto_battlefield(card, owner, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, exile_and_return_tapped, "Whenever The Mighty Thor attacks, exile up to one target nontoken artifact or creature, then return that card to the battlefield tapped under its owner's control.")
            .targets(&[up_to_one(nontoken_artifact_or_creature)]),
        triggered(when(EventMask::ZONE_CHANGE, equipment_you_control_enters), |g, item| effects::draw(g, item, 1), "Whenever an Equipment you control enters, draw a card."),
    ],
    ..oracle::THE_MIGHTY_THOR_JANE_FOSTER
};
