use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::tap_and_stun_then_draw),
    ..oracle::CRYOGENIC_STASIS
};
