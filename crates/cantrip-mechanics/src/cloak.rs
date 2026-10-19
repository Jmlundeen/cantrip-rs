//! Cloak (CR 701.58): "To cloak a card, put it onto the battlefield face down as a 2/2 creature with ward {2}. Turn it
//! face up any time for its mana cost if it's a creature card." Cryptic Coat, Hide in Plain Sight, Ransom Note, Veiled
//! Ascension, Unexplained Absence and Etrata, Deadly Fugitive cloak from a library, Vannifar, Evolved Enigma from a hand,
//! Become Anonymous and Expose the Culprit cards exiled in a face-down pile.
//!
//! What a cloaked permanent is, and turning it face up, are the rules' (`FaceDown::Cloaked`, `intrinsic::CLOAKED`).

use crate::top_of_library;
use cantrip_api::*;

/// `player` cloaks `cards`, one at a time (CR 701.58e), each entering under their control (tapped if `tapped`: Become
/// Anonymous's "They enter tapped"). The cloaked permanents, in that order: a card that did not enter face down (CR
/// 701.58f, 400.7) is not among them.
pub fn cloak(g: &mut dyn Game, player: PlayerId, cards: &[ObjectId], tapped: bool) -> Result<Vec<ObjectId>, Suspend> {
    let mut cloaked = Vec::with_capacity(cards.len());
    for &card in cards {
        if let Some(permanent) = g.put_onto_battlefield_face_down(card, player, FaceDown::Cloaked, tapped)? {
            cloaked.push(permanent);
        }
    }
    Ok(cloaked)
}

/// `player` cloaks the top card of `library`'s owner's library (Cryptic Coat, Ransom Note and Veiled Ascension their
/// own; Etrata, Deadly Fugitive an opponent's): the cloaked permanent, so a card can attach to it, or None with an empty
/// library or a card that did not enter face down.
pub fn cloak_top_of_library(g: &mut dyn Game, player: PlayerId, library: PlayerId) -> Result<Option<ObjectId>, Suspend> {
    let Some(&top) = top_of_library(g, library, 1).first() else { return Ok(None) };
    g.put_onto_battlefield_face_down(top, player, FaceDown::Cloaked, false)
}
