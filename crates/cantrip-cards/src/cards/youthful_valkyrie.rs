use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Angel) }>),
        plus_one_counter_on_this,
        "Whenever another Angel you control enters, put a +1/+1 counter on this creature.",
    )],
    ..oracle::YOUTHFUL_VALKYRIE
};
