use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "As long as an artifact entered the battlefield under your control this turn, this creature can attack as though it didn't
/// have defender" (CR 702.3b, 609.4, 603.6a): only this creature, and only for an artifact that entered under its controller's
/// control this turn - the history keeps each permanent as it entered, so one that has since died or been stolen still counts
/// and one an opponent controlled as it entered does not. It keeps defender for everything else. Defender is a core keyword
/// read from its characteristics.
fn may_attack_after_an_artifact_entered(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    creature == asker.source && history::artifact_entered_under_control_this_turn(q.history(), asker.controller)
}

pub static CARD: CardDef = CardDef {
    attacks_despite_defender: &[may_attack_after_an_artifact_entered],
    ..oracle::MECHAN_SHIELDMATE
};
