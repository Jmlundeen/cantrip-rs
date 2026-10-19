use cantrip_api::*;
use cantrip_oracle as oracle;
/// Enters prepared (CR 722); Peer Review is the same spell printed on Prudent Fateseer.
pub static CARD: CardDef = CardDef {
 enters: Some(cantrip_mechanics::prepare::enters_prepared),
 prepare: Some(&super::prudent_fateseer::PEER_REVIEW),
 ..oracle::FATEHOLD_CHRONOLOGIST
};
