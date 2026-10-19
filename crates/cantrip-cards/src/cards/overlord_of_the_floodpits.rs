use cantrip_api::*;
use cantrip_mechanics::impending;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Draw two cards, then discard a card (CR 121.1, 701.9): the card to discard is chosen from the hand as it stands after
/// both draws.
fn draw_two_then_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    cantrip_mechanics::discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[impending::way(mana("{1}{U}{U}"), "Impending 4\u{2014}{1}{U}{U}")],
    enters: Some(impending::enters::<4>),
    statics: &[impending::NOT_A_CREATURE],
    abilities: &[
        triggered(triggers::THIS_ENTERS_OR_ATTACKS, draw_two_then_discard, "Whenever this permanent enters or attacks, draw two cards, then discard a card."),
        impending::REMOVE_A_TIME_COUNTER,
    ],
    ..oracle::OVERLORD_OF_THE_FLOODPITS
};
