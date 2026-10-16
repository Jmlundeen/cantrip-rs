use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever an opponent draws their second card each turn" (CR 120.3, 603.2): the draw itself, whichever step it
/// falls in, by a drawer who had drawn exactly one card this turn before it (the history holds the draws before the
/// one being matched).
fn opponent_second_draw(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Drew { player, .. } if *player != q.controller(source) && cantrip_mechanics::history::drawn_this_turn(q, *player) == 1 => Fired::PLAIN,
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
