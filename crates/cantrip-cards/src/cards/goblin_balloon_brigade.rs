use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_flying;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{R}"),
            this_gains_flying,
            "{R}: This creature gains flying until end of turn.",
        ),
    ],
    ..oracle::GOBLIN_BALLOON_BRIGADE
};
