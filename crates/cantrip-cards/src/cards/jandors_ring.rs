use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{2}, {T}, Discard the last card you drew this turn: Draw a card." (CR 608.2, 121.1).
fn draw_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

/// The last card the controller drew this turn (CR 701.9a, 118.3).
fn last_card_drawn_this_turn(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.history().this_turn().iter().rev().find_map(|r| match r.event {
        Event::Drew { player, card } if player == asker.controller => Some(card),
        _ => None,
    }) == Some(id)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{2}").and(TAP).and(discard(1, last_card_drawn_this_turn)),
            draw_card,
            "{2}, {T}, Discard the last card you drew this turn: Draw a card.",
        ),
    ],
    ..oracle::JANDORS_RING
};
