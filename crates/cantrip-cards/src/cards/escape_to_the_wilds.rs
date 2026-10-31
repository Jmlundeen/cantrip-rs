use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_until_end_of_next_turn;
use cantrip_oracle as oracle;

/// "Exile the top five cards of your library. You may play cards exiled this way until the end of your next turn.
/// You may play an additional land this turn." The five leave together, face up (CR 406.3), and each may be played,
/// cast or as the land for the turn (CR 305.1), until the cleanup step of its controller's next turn (CR 514.2); a
/// library of fewer exiles what it has. The additional land play (CR 305.2) lasts the turn and need not be one of the
/// exiled cards.
fn resolve(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 5);
    g.end_sentence();
    g.additional_land_plays(item, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], resolve), ..oracle::ESCAPE_TO_THE_WILDS };
