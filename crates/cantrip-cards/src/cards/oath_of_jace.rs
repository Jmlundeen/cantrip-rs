use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, scry, triggers};
use cantrip_oracle as oracle;

/// "Draw three cards, then discard two cards" (CR 121.1, 701.9): the two are chosen from the hand once the three are in it.
fn draw_three_discard_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    discard_chosen(g, item.controller, item.object, 2)?;
    Ok(())
}

/// "Scry X, where X is the number of planeswalkers you control": counted as the ability resolves. With none it is
/// scry 0, which is no scry, and nothing that triggers on one triggers (CR 701.22b, the ruling).
fn scry_planeswalkers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let x = g.permanents_of(you).into_iter().filter(|&p| g.chars(p).is(CardType::Planeswalker)).count();
    scry(g, you, item.object, x)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, draw_three_discard_two, "When Oath of Jace enters, draw three cards, then discard two cards."),
        triggered(triggers::YOUR_UPKEEP, scry_planeswalkers, "At the beginning of your upkeep, scry X, where X is the number of planeswalkers you control."),
    ],
    ..oracle::OATH_OF_JACE
};
