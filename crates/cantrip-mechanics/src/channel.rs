//! Channel (CR 207.2c ability word): an ability activated from a hand by discarding the card. Otawara, Soaring City,
//! Boseiju, Who Endures and Takenuma, Abandoned Mire print its reduction.

use cantrip_api::*;

/// "This ability costs {1} less to activate for each legendary creature you control" (CR 601.2f by way of 602.2b): the
/// channel ability's own cost change (`ActivatedDef::cost_change`), asked only as that ability is activated - from the
/// hand, the only zone it functions in - and counted for the player activating it.
pub fn cheaper_per_legendary_creature(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    let legends = q
        .permanents_of(asker.controller)
        .into_iter()
        .filter(|&id| {
            let chars = q.chars(id);
            chars.is(CardType::Creature) && chars.supertypes.legendary
        })
        .count();
    CostDelta::less(legends as u32)
}
