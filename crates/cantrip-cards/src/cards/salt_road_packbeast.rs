use cantrip_api::*;
use cantrip_mechanics::{affinity, effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this creature enters, draw a card.")],
    own_cost_changes: &[affinity::for_creatures],
    ..oracle::SALT_ROAD_PACKBEAST
};
