use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{G}{G}{G}").and(TAP),
        effects::first_target_gains_flying,
        "{G}{G}{G}, {T}: Target creature gains flying until end of turn.",
    )
    .targets(&[targets::CREATURE])],
    ..oracle::PIXIE_QUEEN
};
