use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{choose, delayed, select};
use cantrip_oracle as oracle;

/// "Exile any number of other nonland permanents you own and control" (CR 608.2d, 701.13): the choice is made as the
/// ability resolves, among the permanents the controller controls and also owns, Yorion excluded, and they are exiled
/// together. "Return those cards to the battlefield at the beginning of the next end step" (CR 603.7): the delayed ability
/// is about exactly the objects exiled (CR 603.7c), and is set up even when none were (nothing then returns).
fn exile_and_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let eligible: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&p| p != item.object && g.obj(p).owner == you).collect();
    let rule = select::cards(select::ANY_NUMBER).with(select::NONLAND);
    let chosen = choose::select(g, you, item.object, Purpose::Exile, eligible, rule)?;
    let exiled = g.exile_all(&chosen);
    delayed::at_next_end_step_about(g, item, 0, &exiled);
    Ok(())
}

/// "Companion - Your starting deck contains at least twenty cards more than the minimum deck size" (CR 702.139a): eighty in
/// Constructed, sixty in Limited (CR 100.2a-b, the rulings).
fn twenty_more_than_the_minimum(_: &dyn Query, deck: &StartingDeck) -> bool {
    deck.cards.len() as u64 >= u64::from(deck.minimum_size) + 20
}

pub static CARD: CardDef = CardDef {
    companion: Some(twenty_more_than_the_minimum),
    abilities: &[triggered(THIS_ENTERS, exile_and_return, "When Yorion enters, exile any number of other nonland permanents you own and control. Return those cards to the battlefield at the beginning of the next end step.")],
    delayed: &[delayed_trigger(delayed::return_exiled_to_battlefield, "Return those cards to the battlefield.")],
    ..oracle::YORION_SKY_NOMAD
};
