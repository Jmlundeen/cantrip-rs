use cantrip_api::*;
use cantrip_mechanics::effects::destroy_all_creatures_no_regen;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: destroy_all_creatures_no_regen }),
    ..oracle::WRATH_OF_GOD
};
