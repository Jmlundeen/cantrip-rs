use cantrip_api::*;
use cantrip_mechanics::{hand, select, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Look at target opponent's hand. You may choose a nonland card from it. If you do, that player exiles that card, then draws a card." (CR 701.20e, 608.2d, 701.13a, 608.2c, 121.1)
fn look_and_swap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    if let Some(card) = hand::look_and_may_choose(g, item, opponent, Purpose::Exile, select::NONLAND_CARD)? {
        g.exile(card);
        g.draw(opponent, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look_and_swap, "When this creature enters, look at target opponent's hand. You may choose a nonland card from it. If you do, that player exiles that card, then draws a card.")
        .targets(&[targets::OPPONENT])],
    ..oracle::SALT_VAMPIRE
};
