use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Discard", "discard your hand?", "If you do, draw a card for each spell you've cast this turn", "Discard, then draw", "Keep your hand");

/// "You may discard your hand. If you do, draw cards equal to the number of spells you've cast this turn" (CR 608.2c-d, 701.9):
/// the controller is asked as it resolves; an empty hand may be discarded too and counts as having done it. Only the spells
/// its controller cast count, not an opponent's, and the count is not changed by the discard.
fn may_discard_hand_to_draw_for_spells(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if effects::may_discard_hand(g, item, &ASK)?.is_some() {
        let spells = g.history().casts_by(player).count();
        g.draw(player, spells as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        may_discard_hand_to_draw_for_spells,
        "At the beginning of your end step, you may discard your hand. If you do, draw cards equal to the number of spells you've cast this turn.",
    )],
    ..oracle::NARSET_JESKAI_WAYMASTER
};
