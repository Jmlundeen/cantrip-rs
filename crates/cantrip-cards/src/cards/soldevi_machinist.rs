use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Abilities of artifacts" are those of artifact permanents (CR 109.2: no "source" or "card"), never of an artifact card in
/// a hand or graveyard (as the like rulings of Smokebraider, Myr Reservoir and Castle Garenbrig say); and not artifact
/// spells, nor the costs of triggered abilities (its ruling; CR 106.6).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::C, 2).only_for(ManaRestriction::ABILITIES_OF_ARTIFACTS), "{T}: Add {C}{C}. Spend this mana only to activate abilities of artifacts.")],
    ..oracle::SOLDEVI_MACHINIST
};
