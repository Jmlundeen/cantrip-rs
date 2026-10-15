//! Affinity for artifacts (CR 702.41a): Emry, Lurker of the Loch and Thoughtcast.

use cantrip_api::*;

/// This spell costs {1} less to cast for each artifact you control.
pub fn for_artifacts(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed) -> i8 {
    if !matches!(costed, Costed::Spell(_)) {
        return 0;
    }
    let artifacts = q.permanents_of(player).into_iter().filter(|&id| q.chars(id).is(CardType::Artifact)).count();
    -(artifacts.min(127) as i8)
}
