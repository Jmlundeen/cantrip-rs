use cantrip_api::*;
use cantrip_mechanics::mana::{enters_tapped_unless_two_or_fewer_other_lands, tap_for_either};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_two_or_fewer_other_lands),
    abilities: &[tap_for_either(Color::Green, Color::White, "{T}: Add {G} or {W}.")],
    ..oracle::RAZORVERGE_THICKET
};
