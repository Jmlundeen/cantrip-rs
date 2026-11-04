use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::storage_land::<{ Mana::U as u8 }>("{T}, Remove any number of storage counters from this land: Add {U} for each storage counter removed this way."),
    enters: Some(mana::enters_tapped),
    may_not_untap: true,
    ..oracle::SAND_SILOS
};
