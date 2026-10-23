//! Exert as a creature attacks (CR 701.43d): "You may exert this creature as it attacks. When you do, ..." - an optional
//! cost to attack (CR 508.1g) and the triggered ability linked to it (CR 607.2h). Glorybringer, Ahn-Crop Crasher and Tah-Crop
//! Elite print it. An exerted permanent won't untap during its controller's next untap step (CR 701.43a); the core exerts
//! it as the cost is paid and owns the trigger (`TriggeredDef::attack_option`). Exerting as a cost of an activated ability
//! (Arena of Glory) is `cantrip_api::EXERT`.

use cantrip_api::*;

/// "You may exert this creature as it attacks. When you do, `effect`": `text` is the paragraph as printed. The ability's
/// targets, if any, are chosen as it is put on the stack, after the attackers are declared; with none legal it is removed
/// from the stack, and exerting was still allowed (Glorybringer's ruling, CR 603.3d).
pub const fn as_it_attacks(effect: Effect, text: &'static str) -> AbilityDef {
    paid_as_it_attacks(AttackOption::Exert, effect, text)
}
