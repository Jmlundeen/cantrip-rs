use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{G}{G}"),
        effects::this_gains_first_strike,
        "{G}{G}: This creature gains first strike until end of turn.",
    )],
    ..oracle::EMERALD_DRAGONFLY
};
