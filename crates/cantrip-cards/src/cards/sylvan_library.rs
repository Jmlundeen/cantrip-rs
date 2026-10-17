use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::effects::may_draw;
use cantrip_mechanics::triggers::YOUR_DRAW_STEP;
use cantrip_oracle as oracle;

static DRAW_TWO: Question = Question::new(
    "Draw",
    "draw two additional cards?",
    "If you do, you choose two cards in your hand drawn this turn, and for each pay 4 life or put it on top of your library.",
    "Draw two",
    "Don't draw",
);

static KEEP: Question = Question::new("PayLife", "pay {cost} to keep {subject}?", "If you don't, it goes on top of your library.", "Pay {cost}", "Put it back")
    .amount(4)
    .no_default();

/// "Cards in your hand drawn this turn": each card the turn's history has the player draw that is still in their hand as the
/// card it was drawn as (CR 400.7: one put back and drawn again is drawn again, once), in the order drawn.
fn in_hand_drawn_this_turn(q: &dyn Query, player: PlayerId) -> Vec<ObjectId> {
    q.history()
        .select(EventQuery::kind(EventMask::DREW).player(player).this_turn())
        .filter_map(|r| if let Event::Drew { card, .. } = r.event { Some(card) } else { None })
        .filter(|&card| is_live(q, card) && q.obj(card).zone == Zone::hand(player))
        .collect()
}

/// "At the beginning of your draw step, you may draw two additional cards. If you do, choose two cards in your hand drawn
/// this turn. For each of those cards, pay 4 life or put the card on top of your library."
/// - The choice to draw two is not offered when the player can't draw two (CR 121.2b, 121.3).
/// - "If you do" is the choice: replaced draws leave fewer cards drawn, and with one the player must choose that one, with
///   none the rest does nothing (its ruling).
/// - Nothing can happen between the draws and the choice (its ruling): the cards are chosen as the ability resolves, two
///   if there are two or more (any drawn earlier this turn among them), and nobody is asked to choose when there is no
///   choice to make.
/// - Each chosen card in turn: 4 life paid keeps it (only a player with 4 life to pay is asked, CR 119.4), else it goes on
///   top of the library, the second one put back above the first.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if !may_draw(g, you, item.object, 2, &DRAW_TWO)? {
        return Ok(());
    }
    let drawn = in_hand_drawn_this_turn(g, you);
    let chosen = if drawn.len() <= 2 { drawn } else { choose::cards(g, you, item.object, Purpose::PayLifeOrPutOnTop(4), drawn, 2, false)? };
    for card in chosen {
        if !g.may_pay(you, item.object, card, life(4), &KEEP)? {
            g.put_into_library(&[card], LibraryPlace::Top);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(YOUR_DRAW_STEP, effect, "At the beginning of your draw step, you may draw two additional cards. If you do, choose two cards in your hand drawn this turn. For each of those cards, pay 4 life or put the card on top of your library.")],
    ..oracle::SYLVAN_LIBRARY
};
