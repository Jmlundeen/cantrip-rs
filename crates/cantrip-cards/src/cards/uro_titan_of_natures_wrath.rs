use cantrip_api::*;
use cantrip_mechanics::{escape, hand};
use cantrip_oracle as oracle;

/// You gain 3 life and draw a card, then you may put a land card from your hand onto the battlefield: one "up to one"
/// choice, asked only when there is a land to put.
fn gain_draw_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.gain_life(you, 3);
    g.draw(you, 1);
    hand::may_put_onto_battlefield(g, you, item.object, |c| c.is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        escape::sacrifice_unless_escaped("When Uro enters, sacrifice it unless it escaped."),
        triggered(cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS, gain_draw_land, "Whenever Uro enters or attacks, you gain 3 life and draw a card, then you may put a land card from your hand onto the battlefield."),
    ],
    ways: &[escape::way(mana("{U}{U}{G}{G}"), 5, "Escape\u{2014}{G}{G}{U}{U}, Exile five other cards from your graveyard")],
    ..oracle::URO_TITAN_OF_NATURES_WRATH
};
