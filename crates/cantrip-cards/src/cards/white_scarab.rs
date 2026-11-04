use cantrip_api::*;
use cantrip_mechanics::{scarab, statics, targets};
use cantrip_oracle as oracle;

const WHITE: u8 = scarab::color(Color::White);

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            scarab::enchanted_gets_while_opponent_controls_color::<WHITE>,
            statics::plus_pt::<2, 2>,
            "Enchanted creature gets +2/+2 as long as an opponent controls a white permanent.",
        ),
    ],
    block_rules: &[scarab::cant_be_blocked_by_color::<WHITE>],
    enchant: Some(targets::CREATURE),
    ..oracle::WHITE_SCARAB
};
