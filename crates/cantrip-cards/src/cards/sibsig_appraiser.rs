use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| library::look_at_top_take_one_rest_to_graveyard(g, item.controller, item.object, 2),
        "When this creature enters, look at the top two cards of your library. Put one of them into your hand and the other into your graveyard.",
    )],
    ..oracle::SIBSIG_APPRAISER
};
