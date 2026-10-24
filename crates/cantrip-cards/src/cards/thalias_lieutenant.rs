use cantrip_api::*;
use cantrip_mechanics::effects::{plus_one_counter_on_each_other_of_subtype_you_control, plus_one_counter_on_this};
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            plus_one_counter_on_each_other_of_subtype_you_control::<{ lands::sub(Subtype::Human) }>,
            "When this creature enters, put a +1/+1 counter on each other Human you control.",
        ),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Human) }>),
            plus_one_counter_on_this,
            "Whenever another Human you control enters, put a +1/+1 counter on this creature.",
        ),
    ],
    ..oracle::THALIAS_LIEUTENANT
};
