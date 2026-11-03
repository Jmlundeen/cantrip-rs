use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}").and(TAP),
        effects::target_player_mills::<2>,
        "{2}, {T}: Target player mills two cards.",
    )
    .targets(&[targets::PLAYER])],
    ..oracle::MILLSTONE
};
