use cantrip_api::*;
use cantrip_mechanics::{effects, prepare, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "This creature enters prepared" (CR 614.1d, 722.3a) and "When this creature enters, surveil 1" (CR 603.6a,
    // 701.25a). Peer Review is the same prepare spell Prudent Fateseer and Fatehold Chronologist print (CR 722.3c): its
    // copy is cast from exile and the creature stops being prepared as it is cast (CR 722.3c).
    enters: Some(prepare::enters_prepared),
    prepare: Some(&super::prudent_fateseer::PEER_REVIEW),
    abilities: &[triggered(triggers::THIS_ENTERS, |g, item| effects::surveil(g, item, 1), "When this creature enters, surveil 1.")],
    ..oracle::SEMESTER_FORESEER
};
