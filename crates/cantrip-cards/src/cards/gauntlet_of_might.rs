use cantrip_api::*;
use cantrip_mechanics::{lands, mana, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
                when_asked(EventMask::TAPPED, mana::a_player_taps_a_land_type_for_mana::<{ lands::sub(Subtype::Mountain) }>),
                mana::that_player_adds_an_additional::<{ Mana::R as u8 }>,
                "Whenever a Mountain is tapped for mana, its controller adds an additional {R}.",
            )
            .mana_ability(TriggeredMana::One(Mana::R)),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            statics::creatures_of_color::<{ Color::Red as u8 }>,
            statics::plus_pt::<1, 1>,
            "Red creatures get +1/+1.",
        ),
    ],
    ..oracle::GAUNTLET_OF_MIGHT
};
