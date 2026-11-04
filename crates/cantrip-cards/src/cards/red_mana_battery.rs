use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &mana::mana_battery::<{ Mana::R as u8 }>("{T}, Remove any number of charge counters from this artifact: Add {R}, then add an additional {R} for each charge counter removed this way."),
    ..oracle::RED_MANA_BATTERY
};
