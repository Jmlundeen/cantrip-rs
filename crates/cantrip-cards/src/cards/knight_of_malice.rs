use cantrip_api::*;
use cantrip_mechanics::statics::{itself, plus_one_zero_while_any_player_controls};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Hexproof from white" (CR 702.11d).
    hexproof_from: HexproofFrom::color(Color::White),
    statics: &[static_ability(
        Layer::PtModify,
        itself,
        plus_one_zero_while_any_player_controls::<{ Color::White as u8 }>,
        "This creature gets +1/+0 as long as any player controls a white permanent.",
    )],
    ..oracle::KNIGHT_OF_MALICE
};
