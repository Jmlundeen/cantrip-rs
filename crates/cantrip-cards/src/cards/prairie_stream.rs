use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_two_or_more_basic_lands),
    ..oracle::PRAIRIE_STREAM
};
