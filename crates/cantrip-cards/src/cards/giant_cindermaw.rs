use cantrip_api::*;
use cantrip_mechanics::statics::players_cant_gain_life;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { cant_gain_life: &[players_cant_gain_life], ..oracle::GIANT_CINDERMAW };
