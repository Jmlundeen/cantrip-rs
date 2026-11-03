use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::statics::{all_of_subtype, base_pt, creature_too};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Type,
            all_of_subtype::<{ lands::sub(Subtype::Forest) }>,
            creature_too,
            "All Forests are 1/1 creatures that are still lands.",
        ),
        static_ability(
                Layer::PtSet,
                all_of_subtype::<{ lands::sub(Subtype::Forest) }>,
                base_pt::<1, 1>,
                "All Forests are 1/1 creatures that are still lands.",
            )
            .continues(),
    ],
    ..oracle::LIVING_LANDS
};
