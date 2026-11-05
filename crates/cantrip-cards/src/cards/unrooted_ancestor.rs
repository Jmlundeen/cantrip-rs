use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}").and(sacrifice(1, targets::filters::another_creature)),
        effects::this_gains_indestructible_then_tap,
        "{1}, Sacrifice another creature: This creature gains indestructible until end of turn. Tap it.",
    )],
    ..oracle::UNROOTED_ANCESTOR
};
