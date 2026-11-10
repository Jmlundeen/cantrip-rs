use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::draw_a_card_then_may_put_a_land_tapped,
        "When this creature enters, draw a card, then you may put a land card from your hand onto the battlefield tapped.",
    )],
    ..oracle::H_E_R_B_I_E_SCOUT_UNIT
};
