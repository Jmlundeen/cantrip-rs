use cantrip_api::*;
use cantrip_mechanics::cost_reduction;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cost_reduction::for_each_noncreature_nonland_card_in_your_graveyard],
    ..oracle::RAVENOUS_COSMOZOAN
};
