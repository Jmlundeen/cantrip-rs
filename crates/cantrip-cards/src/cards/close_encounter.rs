use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, warp};
use cantrip_oracle as oracle;

/// "Close Encounter deals damage equal to the power of the chosen creature or card to target creature": the object the
/// cost chose (`Query::cost_objects`). A creature's power is read as the spell resolves, or as it last existed on the
/// battlefield if it has left (CR 608.2h, the ruling); a card in exile, its power there. A permanent that is no longer a
/// creature has no power, and a negative power is 0 (CR 107.1b): a source dealing 0 deals no damage (CR 120.8). The chosen
/// creature may be the target.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&chosen) = g.cost_objects(item).first() else { return Ok(()) };
    let power = g.chars(chosen).power.unwrap_or(0).max(0);
    damage_target(g, item, 0, power as u32);
    Ok(())
}

/// "A creature you control or a warped creature card you own in exile" (CR 702.185b), among the permanents and the cards in
/// exile the cost chooses from (`Cost::choose`).
fn creature_you_control_or_warped_creature_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    if !q.chars(id).is(CardType::Creature) {
        return false;
    }
    if q.obj(id).zone.is(ZoneKind::Battlefield) {
        q.controller(id) == asker.controller
    } else {
        q.obj(id).owner == asker.controller && warp::warped_card_in_exile(q, id)
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    // "As an additional cost to cast this spell, choose a creature you control or a warped creature card you own in exile"
    // (CR 601.2b, 601.2h): exactly one, and without one the spell can't be cast.
    additional_cost: Some(Cost {
        choose: Some((creature_you_control_or_warped_creature_card, "creature you control or warped creature card you own in exile")),
        ..Cost::NONE
    }),
    ..oracle::CLOSE_ENCOUNTER
};
