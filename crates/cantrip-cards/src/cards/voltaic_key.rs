use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}, {T}"), effects::untap_targets, "{1}, {T}: Untap target artifact.")
        .targets(&[targets::ARTIFACT])],
    ..oracle::VOLTAIC_KEY
};
