use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

/// "This ability costs {X} less to activate, where X is the number of differently named lands you control" (CR 601.2f,
/// 602.2b, 118.7a): a generic reduction, the lands counted as the cost is determined, only for the activating player's own.
fn cost_reduction(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(lands::differently_named_lands(q, asker.controller) as u32)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{4}{U}").and(TAP),
        effects::tap_targets,
        "Strange new worlds \u{2014} {4}{U}, {T}: Tap target creature. This ability costs {X} less to activate, where X is the number of differently named lands you control.",
    )
    .targets(&[targets::CREATURE])
    .cost_change(cost_reduction)],
    ..oracle::SUUS_MAHNA_PRACTITIONER
};
