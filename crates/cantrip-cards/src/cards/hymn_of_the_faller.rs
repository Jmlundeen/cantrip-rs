use cantrip_api::*;
use cantrip_mechanics::{effects, history};
use cantrip_oracle as oracle;

/// "Surveil 1, then you draw a card and lose 1 life. Void — If a nonland permanent left the battlefield this turn or a
/// spell was warped this turn, draw another card": the steps in order (CR 608.2c), the void condition read as the spell
/// resolves, after the surveil, the draw and the life loss.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)?;
    effects::draw_a_card_and_lose_1_life(g, item)?;
    if history::void(g) {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::HYMN_OF_THE_FALLER };
