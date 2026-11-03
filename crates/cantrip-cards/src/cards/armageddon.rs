use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effects::destroy_all_of_type::<{ CardType::Land as u8 }>),
    ..oracle::ARMAGEDDON
};
