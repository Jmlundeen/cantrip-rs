use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { enters: Some(cantrip_mechanics::mana::enters_tapped_unless_pay_two_life), ..oracle::HALLOWED_FOUNTAIN };
