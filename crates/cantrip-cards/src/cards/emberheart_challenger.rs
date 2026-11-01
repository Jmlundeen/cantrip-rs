use cantrip_api::*;
use cantrip_mechanics::library::exile_top_card_play_this_turn;
use cantrip_mechanics::triggers::{PROWESS, VALIANT};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        triggered(
            VALIANT,
            exile_top_card_play_this_turn,
            "Valiant \u{2014} Whenever this creature becomes the target of a spell or ability you control for the first time each turn, exile the top card of your library. Until end of turn, you may play that card.",
        ),
    ],
    ..oracle::EMBERHEART_CHALLENGER
};
