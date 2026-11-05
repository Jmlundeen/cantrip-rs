use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}").and(TAP), |g, item| effects::surveil(g, item, 1), "{2}, {T}: Surveil 1."),
        ability(
            cost("{6}").and(TAP).and(SACRIFICE_THIS),
            effects::destroy_first_target,
            "{6}, {T}, Sacrifice Lunatic Pandora: Destroy target nonland permanent.",
        )
        .targets(&[targets::NONLAND_PERMANENT]),
    ],
    ..oracle::LUNATIC_PANDORA
};
