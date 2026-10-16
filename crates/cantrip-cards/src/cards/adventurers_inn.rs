use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 2), "When this land enters, you gain 2 life."),
    ],
    ..oracle::ADVENTURERS_INN
};
