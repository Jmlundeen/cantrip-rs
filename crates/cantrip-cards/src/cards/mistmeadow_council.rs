use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, effects, lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cost_reduction::less_if_you_control_subtype::<1, { lands::sub(Subtype::Kithkin) }>],
    abilities: &[triggered(triggers::THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this creature enters, draw a card.")],
    ..oracle::MISTMEADOW_COUNCIL
};
