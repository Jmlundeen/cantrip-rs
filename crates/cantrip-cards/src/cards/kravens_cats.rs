use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}{G}"), effects::this_gets::<2, 2>, "{2}{G}: This creature gets +2/+2 until end of turn. Activate only once each turn.").once_each_turn()],
    ..oracle::KRAVENS_CATS
};
