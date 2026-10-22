use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// Search your library for a card, then shuffle and put that card on top. You lose 2 life. Not revealed: the card
/// stays in the library, is left out of the shuffle's result and is placed on top (as Mystical Tutor does).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_top(g, item.controller, item.object, select::CARD, false)?;
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::VAMPIRIC_TUTOR };
