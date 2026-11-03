use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Destroy all Forests." (CR 701.8a)
    spell: spell(&[], effects::destroy_all_of_subtype::<{ lands::sub(Subtype::Forest) }>),
    ..oracle::ACID_RAIN
};
