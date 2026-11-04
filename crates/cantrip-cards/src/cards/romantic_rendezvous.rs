use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], |g, item| effects::discard_then_draw(g, item, 2)),
    ..oracle::ROMANTIC_RENDEZVOUS
};
