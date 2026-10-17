use cantrip_api::*;
use cantrip_mechanics::effects::may_draw;
use cantrip_mechanics::triggers::AN_OPPONENT_DRAWS_A_CARD;
use cantrip_oracle as oracle;

static DRAW_TWO: Question = Question::new("Draw", "draw two cards?", "", "Draw two", "Don't draw");

/// "You may draw two cards": both or neither (its ruling), each trigger asked as it resolves. Not offered when the player
/// can't draw two cards (CR 121.2b, 121.3: "can't draw more than one card each turn" forbids the choice outright).
fn may_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    may_draw(g, item.controller, item.object, 2, &DRAW_TWO)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "Whenever an opponent draws a card": once for each card drawn (its ruling); a draw a replacement effect replaced was
    // not made (CR 614.6), so it triggers nothing.
    abilities: &[triggered(AN_OPPONENT_DRAWS_A_CARD, may_draw_two, "Whenever an opponent draws a card, you may draw two cards.")],
    ..oracle::CONSECRATED_SPHINX
};
