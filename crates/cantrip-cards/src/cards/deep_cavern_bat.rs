use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Look at target opponent's hand. You may exile a nonland card from it until this creature leaves the battlefield"
/// (CR 610.3). Looking at the hand is part of the ability, so the choice is asked even with nothing eligible; the card
/// comes back to the hand it was taken from as soon as the Bat is gone, and is not exiled at all if the Bat already is.
fn look_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if hand.is_empty() {
        return Ok(());
    }
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    let ask = RequestKind::ChooseCards {
        source: item.object,
        purpose: Purpose::Exile,
        cards: hand,
        count: eligible.len().min(1) as u8,
        eligible,
        up_to: true, kind: "", total: None };
    if let Answer::Cards(cards) = g.ask(item.controller, ask)? {
        g.exile_with(&cards, item.object, true);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look_and_exile, "When this creature enters, look at target opponent's hand. You may exile a nonland card from it until this creature leaves the battlefield.")
        .targets(&[targets::OPPONENT])],
    ..oracle::DEEP_CAVERN_BAT
};
