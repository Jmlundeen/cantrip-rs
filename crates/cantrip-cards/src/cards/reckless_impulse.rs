use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_until_end_of_next_turn;
use cantrip_oracle as oracle;

/// "Exile the top two cards of your library. Until the end of your next turn, you may play those cards." Playing covers
/// either a land play or casting a spell (CR 701.18b), for the duration the effect states (CR 611.2a). If the library has
/// fewer than two cards, the effect exiles as many as possible (CR 609.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::RECKLESS_IMPULSE };
