use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_this_turn;
use cantrip_mechanics::triggers::{PROWESS, VALIANT};
use cantrip_oracle as oracle;

/// "Exile the top card of your library. Until end of turn, you may play that card": a land too, under the ordinary land-play
/// rules (CR 305.1).
fn exile_top_and_play_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_this_turn(g, item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        triggered(
            VALIANT,
            exile_top_and_play_it,
            "Valiant \u{2014} Whenever this creature becomes the target of a spell or ability you control for the first time each turn, exile the top card of your library. Until end of turn, you may play that card.",
        ),
    ],
    ..oracle::EMBERHEART_CHALLENGER
};
