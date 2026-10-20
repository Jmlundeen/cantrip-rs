use cantrip_api::*;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{delayed, effects, targets};
use cantrip_oracle as oracle;

/// "Target Vehicle card from your graveyard" (CR 115.1): an artifact card of the Vehicle subtype in the ability
/// controller's own graveyard.
fn vehicle_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact])
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(Subtype::Vehicle))
}

/// "Return target Vehicle card from your graveyard to the battlefield. It gains haste. Return it to its owner's hand at
/// the beginning of your next end step" (CR 603.7): the haste lasts as long as the object does, and the delayed ability
/// is about exactly the object that arrived (CR 603.7c, 400.7). It is set up at once, so it is this turn's end step.
fn reanimate_vehicle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let vehicle = g.put_onto_battlefield(card, item.controller, false)?;
    effects::gain_haste(g, item.object, vehicle);
    delayed::at_your_next_end_step_about(g, item, 0, &[vehicle]);
    Ok(())
}

/// "Return it to its owner's hand": the object the ability is about, only if it is on the battlefield as that object
/// (CR 603.7c, 400.7: one that never arrived, or has left, is not affected), sent to its owner's hand (CR 400.3).
fn return_to_owners_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let created: Vec<ObjectId> = g.event_objects(item).iter().copied().filter(|&id| is_live(g, id) && g.is_on_battlefield(id)).collect();
    g.return_to_hand(&created);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(BEGINNING_OF_YOUR_COMBAT, reanimate_vehicle, "At the beginning of combat on your turn, return target Vehicle card from your graveyard to the battlefield. It gains haste. Return it to its owner's hand at the beginning of your next end step.")
        .targets(&[target(vehicle_in_your_graveyard)])],
    delayed: &[delayed_trigger(return_to_owners_hand, "Return it to its owner's hand.")],
    ..oracle::GREASEFANG_OKIBA_BOSS
};
