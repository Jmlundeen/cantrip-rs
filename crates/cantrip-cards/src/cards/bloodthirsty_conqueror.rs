use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::LIFE_CHANGE, cantrip_mechanics::triggers::opponent_loses_life), cantrip_mechanics::effects::gain_that_much_life, "Whenever an opponent loses life, you gain that much life. (Damage causes loss of life.)")],
    ..oracle::BLOODTHIRSTY_CONQUEROR
};
