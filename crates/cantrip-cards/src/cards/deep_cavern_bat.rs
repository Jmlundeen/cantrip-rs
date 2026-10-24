use cantrip_api::*;
use cantrip_mechanics::{hand, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Look at target opponent's hand. You may exile a nonland card from it until this creature leaves the battlefield"
/// (CR 610.3). Looking at the hand is part of the ability, so the choice is asked even with nothing eligible; the card
/// comes back to the hand it was taken from as soon as the Bat is gone, and is not exiled at all if the Bat already is.
fn look_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    if let Some(card) = hand::look_and_may_choose(g, item, opponent, Purpose::Exile, "nonland", |q, c| !q.chars(c).is(CardType::Land))? {
        g.exile_with(&[card], g.linked(item), true);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look_and_exile, "When this creature enters, look at target opponent's hand. You may exile a nonland card from it until this creature leaves the battlefield.")
        .targets(&[targets::OPPONENT])],
    ..oracle::DEEP_CAVERN_BAT
};
