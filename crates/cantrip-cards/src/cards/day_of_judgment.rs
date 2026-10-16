use cantrip_api::*;
use cantrip_mechanics::effects::destroy_all_creatures;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_creatures),
    ..oracle::DAY_OF_JUDGMENT
};
