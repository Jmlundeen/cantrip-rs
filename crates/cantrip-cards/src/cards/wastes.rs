use cantrip_api::*;
use cantrip_oracle as oracle;

/// Wastes is a basic land with no land type (CR 205.3i): it does not have the intrinsic mana ability a basic land type gives, so its own
/// "{T}: Add {C}." is printed on it.
pub static CARD: CardDef = CardDef { abilities: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")], ..oracle::WASTES };
