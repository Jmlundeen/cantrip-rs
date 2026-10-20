//! Channel (CR 207.2c ability word): an ability activated from a hand by discarding the card. Otawara, Soaring City,
//! Boseiju, Who Endures and Takenuma, Abandoned Mire print its reduction.

use cantrip_api::*;

/// "This ability costs {1} less to activate for each legendary creature you control" (CR 601.2f): the card's own change
/// to the cost of its channel ability, which functions only from the hand.
pub fn cheaper_per_legendary_creature(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Ability(card, _) = costed else { return CostDelta::NONE };
    if !q.obj(card).zone.is(ZoneKind::Hand) {
        return CostDelta::NONE;
    }
    let legends = q
        .permanents_of(player)
        .into_iter()
        .filter(|&id| {
            let chars = q.chars(id);
            chars.is(CardType::Creature) && chars.supertypes.legendary
        })
        .count();
    CostDelta::less(legends.min(255) as u8)
}
