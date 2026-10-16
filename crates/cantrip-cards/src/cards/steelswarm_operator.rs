use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::U).only_for(ManaRestriction::ARTIFACT_SPELLS), "{T}: Add {U}. Spend this mana only to cast an artifact spell."),
        mana_ability(TAP, add_n(Mana::U, 2).only_for(ManaRestriction::ARTIFACT_ABILITIES), "{T}: Add {U}{U}. Spend this mana only to activate abilities of artifact sources."),
    ],
    ..oracle::STEELSWARM_OPERATOR
};
