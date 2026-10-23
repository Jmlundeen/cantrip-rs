//! Companion (CR 702.139): "Companion—[Condition]", a keyword ability that functions outside the game. Lurrus of the
//! Dream-Den, Zirda, the Dawnwaker, Lutri, the Spellchaser and Yorion, Sky Nomad print it.
//!
//! A card with companion sets `companion: Some(condition)`, `condition` a `CompanionCondition` over the starting deck
//! (`StartingDeck`, CR 702.139b). The core does the rest: before the game begins, each player in turn order may reveal one
//! card with companion they own from outside the game whose condition their starting deck fulfils (CR 103.2b), and once
//! during the game, with priority in a main phase of their turn and the stack empty, they may pay {3} to put it into their
//! hand (CR 116.2g, a special action). The condition is a card's own; what several print alike is here.

use cantrip_api::*;

use crate::targets::is_permanent_card;

/// "Each permanent card in your starting deck [has the quality]" (CR 110.4a): Lurrus of the Dream-Den ("has mana value 2 or
/// less") and Zirda, the Dawnwaker ("has an activated ability") print it. Each card is read as the card it is in the deck - a
/// modal double-faced card as its front face (CR 712.8a), an {X} in a mana cost as 0 (CR 202.3e).
pub fn each_permanent_card(q: &dyn Query, deck: &StartingDeck, quality: impl Fn(ObjectId) -> bool) -> bool {
    deck.cards.iter().all(|&card| !is_permanent_card(&q.chars(card)) || quality(card))
}
