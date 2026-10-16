use cantrip_api::*;
use cantrip_mechanics::shuffle_hands_and_graveyards_in_and_draw_seven;
use cantrip_oracle as oracle;

/// "Each player shuffles their hand and graveyard into their library, then draws seven cards. If it's your turn, end the
/// turn." Day's Undoing is still on the stack as it resolves, so it is not shuffled in. On its controller's own turn the
/// turn ends (CR 724.1), and the abilities that triggered while the cards moved and were drawn cease to exist (724.1a);
/// on another player's turn (cast with flash, or as an effect resolves) the turn goes on.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    shuffle_hands_and_graveyards_in_and_draw_seven(g);
    if g.state().turn.active == item.controller {
        g.end_the_turn()?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::DAYS_UNDOING };
