use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &mana::mana_battery::<{ Mana::W as u8 }>("{T}, Remove any number of charge counters from this artifact: Add {W}, then add an additional {W} for each charge counter removed this way."),
    ..oracle::WHITE_MANA_BATTERY
};
