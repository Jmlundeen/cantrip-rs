use cantrip_api::*;
use cantrip_mechanics::{scarab, statics, targets};
use cantrip_oracle as oracle;

const BLUE: u8 = scarab::color(Color::Blue);

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            scarab::enchanted_gets_while_opponent_controls_color::<BLUE>,
            statics::plus_pt::<2, 2>,
            "Enchanted creature gets +2/+2 as long as an opponent controls a blue permanent.",
        ),
    ],
    block_rules: &[scarab::cant_be_blocked_by_color::<BLUE>],
    enchant: Some(targets::CREATURE),
    ..oracle::BLUE_SCARAB
};
