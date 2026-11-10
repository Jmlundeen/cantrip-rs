use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOU_GAIN_LIFE};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Strange new worlds — When Christine Chapel enters, you gain life equal to the number of differently named lands you
    // control" (CR 201.2a, 608.2c): counted as the ability resolves, so a land that entered with it counts, and two lands
    // with the same name count once. The shared body. The life gained is one event, so the second ability triggers once
    // (CR 119.3, 603.2c).
    abilities: &[
        triggered(
            THIS_ENTERS,
            effects::gain_life_for_differently_named_lands,
            "Strange new worlds \u{2014} When Christine Chapel enters, you gain life equal to the number of differently named lands you control.",
        ),
        triggered(
            YOU_GAIN_LIFE,
            effects::two_plus_one_counters_on_this,
            "Whenever you gain life, put two +1/+1 counters on Christine Chapel.",
        ),
    ],
    ..oracle::CHRISTINE_CHAPEL_COMBAT_MEDIC
};
