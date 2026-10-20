use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_until_end_of_next_turn;
use cantrip_oracle as oracle;

/// "Exile the top two cards of your library. Until the end of your next turn, you may play those cards." The two leave
/// together, face up (CR 101.4); each is the caster's to play, a land as the turn's land play, until the cleanup step of
/// their next turn (CR 514.2). A library of fewer exiles what it has.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::WRENNS_RESOLVE };
