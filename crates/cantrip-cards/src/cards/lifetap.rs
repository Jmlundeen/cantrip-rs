use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(
                EventMask::TAPPED,
                triggers::subtype_an_opponent_controls_becomes_tapped::<{ lands::sub(Subtype::Forest) }>,
            ),
            cantrip_mechanics::effects::gain_one_life,
            "Whenever a Forest an opponent controls becomes tapped, you gain 1 life.",
        ),
    ],
    ..oracle::LIFETAP
};
