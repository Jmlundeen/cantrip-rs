use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Exactly two target lands (CR 601.2c, 602.2b): with fewer than two lands on the battlefield the ability cannot be activated.
    // Untapped are the ones still legal targets as the ability resolves (CR 608.2b).
    abilities: &[ability(cost("{T}"), effects::untap_targets, "{T}: Untap two target lands.").targets(&[targets::LAND.between(2, 2)])],
    ..oracle::ARGOTHIAN_ELDER
};
