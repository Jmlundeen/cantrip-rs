use cantrip_api::*;
use cantrip_mechanics::attach::equipment_you_control_enters;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Exile up to one target nontoken artifact or creature, then return that card to the battlefield tapped under its owner's
/// control": exiled, then returned as a new object (CR 400.7), entering tapped (CR 110.5b). The target is a card, never a token.
/// A merged permanent is exiled as each of its cards, and each returns (CR 730.3c), all together: one event, so what watches
/// "one or more" creatures enter sees them once (CR 101.4, 603.6a).
fn exile_and_return_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let returning: Vec<(ObjectId, PlayerId, bool)> = g
        .exile_all(&[permanent])
        .into_iter()
        .filter(|&card| g.obj(card).zone.is(ZoneKind::Exile))
        .map(|card| (card, g.obj(card).owner, true))
        .collect();
    g.put_all_onto_battlefield_as(&returning)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, exile_and_return_tapped, "Whenever The Mighty Thor attacks, exile up to one target nontoken artifact or creature, then return that card to the battlefield tapped under its owner's control.")
            .targets(&[up_to_one(targets::nontoken_artifact_or_creature_filter)]),
        triggered(when(EventMask::ZONE_CHANGE, equipment_you_control_enters), |g, item| effects::draw(g, item, 1), "Whenever an Equipment you control enters, draw a card."),
    ],
    ..oracle::THE_MIGHTY_THOR_JANE_FOSTER
};
