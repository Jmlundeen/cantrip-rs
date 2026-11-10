use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask(EventMask::BLOCKS.0 | EventMask::BECAME_BLOCKED.0), triggers::this_blocks_or_becomes_blocked),
        effects::pump_self::<2, 2>,
        "Whenever this creature blocks or becomes blocked, it gets +2/+2 until end of turn.",
    )],
    ..oracle::CHUB_TOAD
};
