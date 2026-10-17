//! Looking into a player's hand and taking a card from it (CR 701.9, 701.17).

use crate::choose;
use cantrip_api::*;

/// "Target player reveals their hand. You choose a card from it [that passes `eligible`]": the hand is revealed, and the
/// ability's controller chooses one eligible card. The revealing is the looking; when nothing in the hand is eligible
/// there is no choice to make, so none is asked (CR 608.2d), and nothing at all for an empty hand. The cards chosen are left for the
/// caller to discard or exile. Duress, Thoughtseize and Inquisition of Kozilek print it (`purpose` is the question's: a
/// discard (`Purpose::TakeFromHand`) or an exile (`Purpose::Exile`): the question says which, to whoever is choosing).
pub fn choose_from_revealed_hand(
    g: &mut dyn Game,
    item: &StackItem,
    player: PlayerId,
    purpose: Purpose,
    eligible: fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    choose_from_revealed_hand_of_kind(g, item, player, purpose, "", eligible)
}

/// `choose_from_revealed_hand` for a card that says what it may take ("nonland", "artifact or creature"): `kind` is its own
/// words, which the question says to the chooser. Intimidation Tactics, Thoughtseize and Grief print one.
pub fn choose_from_revealed_hand_of_kind(
    g: &mut dyn Game,
    item: &StackItem,
    player: PlayerId,
    purpose: Purpose,
    kind: &'static str,
    eligible: fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    if hand.is_empty() {
        return Ok(Vec::new());
    }
    g.reveal(player, &hand);
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| eligible(&*g, c)).collect();
    Ok(choose::cards_from_kind(g, item.controller, item.object, purpose, kind, hand, eligible, 1, false)?.into_iter().take(1).collect())
}

/// `choose_from_revealed_hand`, then the player discards what was chosen, together (CR 701.9a). The cards discarded.
pub fn discard_chosen_from_revealed_hand(
    g: &mut dyn Game,
    item: &StackItem,
    player: PlayerId,
    purpose: Purpose,
    eligible: fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    discard_chosen_from_revealed_hand_of_kind(g, item, player, purpose, "", eligible)
}

/// `discard_chosen_from_revealed_hand` for a card that says what may be taken ("nonland"): Thoughtseize, Grief and Humiliate.
pub fn discard_chosen_from_revealed_hand_of_kind(
    g: &mut dyn Game,
    item: &StackItem,
    player: PlayerId,
    purpose: Purpose,
    kind: &'static str,
    eligible: fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let chosen = choose_from_revealed_hand_of_kind(g, item, player, purpose, kind, eligible)?;
    g.discard_together(&chosen);
    Ok(chosen)
}

/// "[Target opponent] reveals their hand. You choose a nonland card from it. That player discards that card." (CR 701.9a): the
/// reveal and the choice are asked even when no nonland card is there to take, as Duress is. Grief, Humiliate and
/// Down, Down to Goblin-town print it.
pub fn discard_chosen_nonland_from_revealed_hand(g: &mut dyn Game, item: &StackItem, player: PlayerId) -> Result<Vec<ObjectId>, Suspend> {
    discard_chosen_from_revealed_hand_of_kind(g, item, player, Purpose::TakeFromHand, "nonland", |q, c| !q.chars(c).is(CardType::Land))
}
