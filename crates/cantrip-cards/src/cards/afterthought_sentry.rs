use cantrip_api::*;
use cantrip_mechanics::effects::{exile_first_target, this_gains_flying};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}"), this_gains_flying, "{2}: This creature gains flying until end of turn."),
        triggered(THIS_ATTACKS, exile_first_target, "Whenever this creature attacks, exile up to one target card from a graveyard.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
    ],
    ..oracle::AFTERTHOUGHT_SENTRY
};
