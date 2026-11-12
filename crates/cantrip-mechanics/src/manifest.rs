//! Manifest (CR 701.40): "To manifest a card, put it onto the battlefield face down as a 2/2 creature. ... Turn it face
//! up any time for its mana cost if it's a creature card." Reality Shift manifests the top card of a library, and
//! manifest dread (`manifest_dread`) manifests one of the top two.
//!
//! What a manifested permanent is, and turning it face up, are the rules' (`FaceDown::Manifested`).

use crate::top_of_library;
use cantrip_api::*;

/// `player` manifests `card` (CR 701.40a): it enters under their control face down, untapped. The manifested permanent,
/// or None when the card did not enter face down (CR 701.40f: a rule or effect kept it from entering).
pub fn manifest(g: &mut dyn Game, player: PlayerId, card: ObjectId) -> Result<Option<ObjectId>, Suspend> {
    g.put_onto_battlefield_face_down(card, player, FaceDown::Manifested, false)
}

/// `player` manifests the top card of their own library (CR 701.40a): the manifested permanent, or None with an empty
/// library (nothing happens) or a card that did not enter face down.
pub fn manifest_top_of_library(g: &mut dyn Game, player: PlayerId) -> Result<Option<ObjectId>, Suspend> {
    let Some(&top) = top_of_library(g, player, 1).first() else { return Ok(None) };
    manifest(g, player, top)
}
