//! Looking into a player's hand and taking a card from it (CR 701.9, 701.17).

use crate::choose;
use cantrip_api::*;

/// "Target player reveals their hand. You choose a card from it [that passes `eligible`]": the hand is revealed, and the
/// ability's controller chooses one eligible card. The revealing is the looking; when nothing in the hand is eligible
/// there is no choice to make, so none is asked (CR 608.2d), and nothing at all for an empty hand. The cards chosen are left for the
/// caller to discard or exile. Duress, Thoughtseize and Inquisition of Kozilek print it (`purpose` is the question's: a
/// discard, or `Purpose::TakeFromHand`).
pub fn choose_from_revealed_hand(
    g: &mut dyn Game,
    item: &StackItem,
    player: PlayerId,
    purpose: Purpose,
    eligible: fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    if hand.is_empty() {
        return Ok(Vec::new());
    }
    g.reveal(player, &hand);
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| eligible(&*g, c)).collect();
    Ok(choose::card_from(g, item.controller, item.object, purpose, hand, eligible)?.into_iter().collect())
}

/// `choose_from_revealed_hand`, then the player discards what was chosen, together (CR 701.9a). The cards discarded.
pub fn discard_chosen_from_revealed_hand(
    g: &mut dyn Game,
    item: &StackItem,
    player: PlayerId,
    purpose: Purpose,
    eligible: fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let chosen = choose_from_revealed_hand(g, item, player, purpose, eligible)?;
    g.discard_together(&chosen);
    Ok(chosen)
}
