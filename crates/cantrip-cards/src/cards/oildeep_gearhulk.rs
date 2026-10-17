use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS, ward};
use cantrip_mechanics::{hand, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, look at target player's hand. You may choose a card from it. If you do, that player discards that
/// card, then draws a card." (CR 701.9a, 121.1) The player may be its own controller. The draw follows only a choice; the
/// chosen card is discarded by that player (the discard can be replaced), and an empty hand is nothing to choose from.
fn look_and_swap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    if let Some(card) = hand::look_and_may_choose(g, item, player, Purpose::TakeFromHand, "", |_, _| true)? {
        g.discard_together(&[card]);
        g.draw(player, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)"),
        triggered(THIS_ENTERS, look_and_swap, "When this creature enters, look at target player's hand. You may choose a card from it. If you do, that player discards that card, then draws a card.")
            .targets(&[targets::PLAYER]),
    ],
    ..oracle::OILDEEP_GEARHULK
};
