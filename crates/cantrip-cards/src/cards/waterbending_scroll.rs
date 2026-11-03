use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, effects, lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{6}, {T}"),
        |g, item| effects::draw(g, item, 1),
        "{6}, {T}: Draw a card. This ability costs {1} less to activate for each Island you control.",
    )
    .cost_change(cost_reduction::ability_cheaper_per_subtype_you_control::<{ lands::sub(Subtype::Island) }>)],
    ..oracle::WATERBENDING_SCROLL
};
