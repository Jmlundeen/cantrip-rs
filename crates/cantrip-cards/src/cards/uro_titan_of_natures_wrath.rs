use cantrip_api::*;
use cantrip_mechanics::{choose, escape};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// You gain 3 life and draw a card, then you may put a land card from your hand onto the battlefield: one "up to one"
/// choice, asked only when there is a land to put.
fn gain_draw_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.gain_life(you, 3);
    g.draw(you, 1);
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    for land in choose::cards_from(g, you, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)? {
        g.put_onto_battlefield(land, you, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        escape::sacrifice_unless_escaped("When Uro enters, sacrifice it unless it escaped."),
        triggered(THIS_ENTERS, gain_draw_land, "Whenever Uro enters, you gain 3 life and draw a card, then you may put a land card from your hand onto the battlefield."),
        triggered(cantrip_mechanics::triggers::THIS_ATTACKS, gain_draw_land, "Whenever Uro attacks, you gain 3 life and draw a card, then you may put a land card from your hand onto the battlefield."),
    ],
    ways: &[escape::way(mana(0, 0, 2, 0, 0, 2), 5, "Escape\u{2014}{G}{G}{U}{U}, Exile five other cards from your graveyard")],
    ..oracle::URO_TITAN_OF_NATURES_WRATH
};
