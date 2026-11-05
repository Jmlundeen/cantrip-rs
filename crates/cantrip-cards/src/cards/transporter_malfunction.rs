use cantrip_api::*;
use cantrip_mechanics::beam_me_up;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<3>),
    ways: &[beam_me_up::way(mana("{3}{R}{R}{R}"), "Beam me up {3}{R}{R}{R}")],
    ..oracle::TRANSPORTER_MALFUNCTION
};
