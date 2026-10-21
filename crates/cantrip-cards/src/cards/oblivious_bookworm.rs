use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, effects::may_draw, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

const ASK_LOOT: Question = Question::new("Draw", "draw a card?", "If you do, discard a card", "Draw and discard", "Don't draw");
const ASK_DRAW: Question =
    Question::new("Draw", "draw a card?", "No discard: a permanent entered face down under your control or you turned one face up this turn", "Draw", "Don't draw");

/// "A permanent entered the battlefield face down under your control this turn or you turned a permanent face up this
/// turn" (CR 708.3, 708.7): asked of this turn's history, so a permanent that entered face down counts though it has since
/// been turned face up or left, and one turned face up though it has since left (the ruling: neither needs to be on the
/// battlefield). "Under your control" is its controller as it entered; "you turned" is the player who did it, by the special
/// action or by an effect they controlled - not an opponent turning up a permanent of theirs.
fn face_down_or_turned_up_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().this_turn().iter().any(|r| match r.event {
        Event::EnteredFaceDown { controller, .. } => controller == player,
        Event::TurnedFaceUp { player: by, .. } => by == player,
        _ => false,
    })
}

/// "You may draw a card. If you do, discard a card unless ..." (CR 608.2c): "if you do" is whether a card was drawn, so an
/// empty library discards nothing, and a player who can't draw is not asked (CR 121.3); the condition is read as the ability resolves (the ruling: too late to manifest or
/// turn a permanent face up once it has begun to resolve). The question says whether the discard will follow.
fn end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let ask = if face_down_or_turned_up_this_turn(g, player) { &ASK_DRAW } else { &ASK_LOOT };
    let before = g.history().drawn(player);
    if !may_draw(g, player, item.object, 1, ask)? {
        return Ok(());
    }
    if g.history().drawn(player) > before && !face_down_or_turned_up_this_turn(g, player) {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(YOUR_END_STEP, end_step, "At the beginning of your end step, you may draw a card. If you do, discard a card unless a permanent entered the battlefield face down under your control this turn or you turned a permanent face up this turn.")],
    ..oracle::OBLIVIOUS_BOOKWORM
};
