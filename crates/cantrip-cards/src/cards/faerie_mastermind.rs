use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever an opponent draws their second card each turn" (CR 120.3, 603.2): the draw itself, whichever step it
/// falls in, by an opponent of this creature's controller.
fn opponent_second_draw(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match cantrip_mechanics::history::second_card_drawn_this_turn(q, event) {
        Some(player) if player != q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}


/// {3}{U}: Each player draws a card, in turn order from the active player (CR 101.4).
fn everyone_draws(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    for player in g.state().apnap().collect::<Vec<_>>() {
        g.draw(player, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::DREW, opponent_second_draw), |g, item| cantrip_mechanics::effects::draw(g, item, 1), "Whenever an opponent draws their second card each turn, you draw a card."),
        ability(cost("{3}{U}"), everyone_draws, "{3}{U}: Each player draws a card."),
    ],
    ..oracle::FAERIE_MASTERMIND
};
