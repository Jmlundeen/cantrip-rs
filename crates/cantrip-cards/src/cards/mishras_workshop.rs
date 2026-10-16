use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::C, 3).only_for(ManaRestriction::ARTIFACT_SPELLS), "{T}: Add {C}{C}{C}. Spend this mana only to cast artifact spells.")],
    ..oracle::MISHRAS_WORKSHOP
};
