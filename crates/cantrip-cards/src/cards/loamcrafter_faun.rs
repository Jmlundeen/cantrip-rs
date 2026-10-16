use cantrip_api::*;
use cantrip_mechanics::{choose, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// You may discard one or more land cards, together (CR 701.9): chosen from among the hand, only land cards eligible.
/// When you do (CR 603.12), the reflexive ability triggers counting them: "that many" is the number of land cards
/// discarded, and none discarded is not doing it.
fn discard_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let lands: Vec<ObjectId> = hand.iter().copied().filter(|&card| g.chars(card).is(CardType::Land)).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::Discard, "land", hand, lands, u8::MAX, true)?;
    let discarded = g.discard_together(&chosen);
    if !discarded.is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger_counted(item.object, you, AbilityRef::delayed(this, 0), Target::None, discarded.len() as u16);
    }
    Ok(())
}

/// Return up to that many target nonland permanent cards from your graveyard to your hand, those still legal
/// (CR 608.2b), together.
fn return_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.return_to_hand(&targets::legal_objects(g, item));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, discard_lands, "When this creature enters, you may discard one or more land cards. When you do, return up to that many target nonland permanent cards from your graveyard to your hand.")],
    delayed: &[delayed_trigger(return_cards, "When you do, return up to that many target nonland permanent cards from your graveyard to your hand.")
        .targets(&[targets::NONLAND_PERMANENT_CARD_IN_YOUR_GRAVEYARD.between(0, TargetSpec::COUNTED)])],
    ..oracle::LOAMCRAFTER_FAUN
};
