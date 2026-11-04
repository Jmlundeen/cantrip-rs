use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}"), plus_one_counter_on_this, "{3}: Put a +1/+1 counter on this creature. Activate only as a sorcery.").sorcery_speed()],
    ..oracle::INTREPID_TENDERFOOT
};
