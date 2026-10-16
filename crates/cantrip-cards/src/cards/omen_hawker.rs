use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_each(&[Mana::C, Mana::U]).only_for(ManaRestriction::ABILITIES), "{T}: Add {C}{U}. Spend this mana only to activate abilities.")],
    ..oracle::OMEN_HAWKER
};
