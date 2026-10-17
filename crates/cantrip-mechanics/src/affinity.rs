//! Affinity (CR 702.41a): affinity for artifacts (Emry, Lurker of the Loch and Thoughtcast) and for a basic land type
//! (Sapling Nursery's Forests).

use cantrip_api::*;

/// This spell costs {1} less to cast for each artifact you control.
pub fn for_artifacts(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    if !matches!(costed, Costed::Spell(_)) {
        return CostDelta::NONE;
    }
    let artifacts = q.permanents_of(player).into_iter().filter(|&id| q.chars(id).is(CardType::Artifact)).count();
    CostDelta::less(artifacts.min(255) as u8)
}

/// "Affinity for [basic land type]es" (CR 702.41a): this spell costs {1} less to cast for each land of that type you control,
/// `TYPE` being `lands::sub(Subtype::Forest)` and so on. The type is a word of the spell's rules text, read through its words
/// (CR 612.2), and a land has it when the layers made it that type (a Forest is any land with the subtype, basic or not).
/// Sapling Nursery prints affinity for Forests.
pub fn for_land_type<const TYPE: u16>(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    if !matches!(costed, Costed::Spell(_)) {
        return CostDelta::NONE;
    }
    let wanted = crate::lands::word(q, source, TYPE);
    let lands = q.permanents_of(player).into_iter().filter(|&id| q.chars(id).is(CardType::Land) && q.chars(id).subtypes.has(wanted)).count();
    CostDelta::less(lands.min(255) as u8)
}
