use cantrip_api::*;
use cantrip_mechanics::effects::this_gets_and_gains_menace;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{R}"), this_gets_and_gains_menace::<1, 0>, "{1}{R}: This creature gets +1/+0 and gains menace until end of turn.")],
    ..oracle::FLAME_CHAIN_MAULER
};
