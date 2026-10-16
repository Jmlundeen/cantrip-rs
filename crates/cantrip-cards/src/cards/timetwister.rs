use cantrip_api::*;
use cantrip_mechanics::shuffle_hands_and_graveyards_in_and_draw_seven;
use cantrip_oracle as oracle;

/// It is still on the stack while it resolves, so it goes to the graveyard after the shuffle, not into the library.
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    shuffle_hands_and_graveyards_in_and_draw_seven(g);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TIMETWISTER };
