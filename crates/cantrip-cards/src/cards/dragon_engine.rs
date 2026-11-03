use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}"), effects::this_gets::<1, 0>, "{2}: This creature gets +1/+0 until end of turn.")],
    ..oracle::DRAGON_ENGINE
};
