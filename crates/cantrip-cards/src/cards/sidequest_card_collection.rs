use cantrip_api::*;
use cantrip_mechanics::{crew::crew, discard_chosen, effects, triggers};
use cantrip_oracle as oracle;

/// "If eight or more cards are in your graveyard" (CR 603.4, 404.1): an intervening "if", asked as the end step begins and again
/// as the ability resolves, of the graveyard of the ability's controller.
fn eight_or_more_cards_in_your_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::graveyard(asker.controller)).len() >= 8
}

/// "Draw three cards, then discard two cards" (CR 121.1, 701.9): the two are chosen from the hand once the three are in it.
fn draw_three_discard_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    discard_chosen(g, item.controller, item.object, 2)?;
    Ok(())
}

/// "Flying" (CR 702.9, from the content) and "Crew 1" (CR 702.122a) of the back face, Magicked Card, a 4/4 Vehicle.
pub static BACK: CardDef = CardDef {
    abilities: &[crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)")],
    ..oracle::MAGICKED_CARD
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, draw_three_discard_two, "When this enchantment enters, draw three cards, then discard two cards."),
        triggered(triggers::YOUR_END_STEP, effects::transform_self, "At the beginning of your end step, if eight or more cards are in your graveyard, transform this enchantment.")
            .condition(eight_or_more_cards_in_your_graveyard),
    ],
    back: Some(&BACK),
    ..oracle::SIDEQUEST_CARD_COLLECTION
};
