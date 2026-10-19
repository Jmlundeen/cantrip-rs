use cantrip_api::*;
use cantrip_mechanics::{crew::crew, effects::may_draw_then_discard, triggers::THIS_ATTACKS_OR_BLOCKS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS_OR_BLOCKS, may_draw_then_discard, "Whenever this Vehicle attacks or blocks, you may draw a card. If you do, discard a card."),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::SMUGGLERS_COPTER
};
