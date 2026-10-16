use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Abilities of artifacts" is the older wording of "abilities of artifact sources" (CR 106.6; its ruling: not on artifact
/// spells, nor on the costs of triggered abilities).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::C, 2).only_for(ManaRestriction::ARTIFACT_ABILITIES), "{T}: Add {C}{C}. Spend this mana only to activate abilities of artifacts.")],
    ..oracle::SOLDEVI_MACHINIST
};
