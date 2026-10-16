use cantrip_api::*;
use cantrip_mechanics::{affinity, effects};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_artifacts],
    spell: spell(&[], |g, item| effects::draw(g, item, 2)),
    ..oracle::THOUGHTCAST
};
