use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect: effects::draw::<3> }), ..oracle::TREASURE_CRUISE };
