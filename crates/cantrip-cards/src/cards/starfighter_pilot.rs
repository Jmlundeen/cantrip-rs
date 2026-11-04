use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_BECOMES_TAPPED;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_BECOMES_TAPPED, |g, item| effects::surveil(g, item, 1), "Whenever this creature becomes tapped, surveil 1.")],
    ..oracle::STARFIGHTER_PILOT
};
