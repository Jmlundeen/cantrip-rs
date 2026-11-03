use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Destroy all Plains." (CR 701.8a)
    spell: spell(&[], effects::destroy_all_of_subtype::<{ lands::sub(Subtype::Plains) }>),
    ..oracle::FLASHFIRES
};
