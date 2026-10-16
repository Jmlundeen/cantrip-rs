use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_any_color(1).only_for(ManaRestriction::LegendaryUncounterable), "{T}: Add one mana of any color. Spend this mana only to cast a legendary spell, and that spell can't be countered."),
    ],
    ..oracle::DELIGHTED_HALFLING
};
