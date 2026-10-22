use cantrip_api::*;
use cantrip_mechanics::{library::exile_top_play_until_end_of_next_turn, triggers};
use cantrip_oracle as oracle;

/// "When this creature dies, exile the top card of your library. You may play it until the end of your next turn."
/// "You" is the controller the creature last had (CR 603.10a): it plays the card, a land under the land-play rules and a spell with
/// all its costs and timing (CR 305.1, 601.3), until the cleanup step of their next turn. An empty library exiles nothing.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_DIES,
        exile_top,
        "When this creature dies, exile the top card of your library. You may play it until the end of your next turn.",
    )],
    ..oracle::CLOCKWORK_PERCUSSIONIST
};
