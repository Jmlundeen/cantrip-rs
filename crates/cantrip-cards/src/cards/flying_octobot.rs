use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

/// Flying is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Villain) }>),
        effects::plus_one_counter_on_this,
        "Whenever another Villain you control enters, put a +1/+1 counter on this creature. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)],
    ..oracle::FLYING_OCTOBOT
};
