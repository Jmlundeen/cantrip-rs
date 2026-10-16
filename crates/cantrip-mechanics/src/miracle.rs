//! Miracle [cost] (CR 702.94a) given to cards in a hand by a static ability of a permanent: "each instant and sorcery card
//! in your hand has miracle {2}" (Lorehold, the Historian), "nonland cards in your hand have miracle {0}" (Molecule Man),
//! "each enchantment card in your hand has miracle" (Aminatou, Veil Piercer).
//!
//! The reveal as the card is drawn and the linked triggered ability are the core's (`ops::draw`, `MIRACLE_ABILITY`). A
//! card that prints miracle sets `CardDef::miracle`; a permanent that gives it lists a `GrantedMiracle` in its
//! `granted_miracles`, whose `has` asks `in_your_hand` and then the card's own characteristics.

use cantrip_api::*;

/// "In your hand": the card is in the hand of the granting permanent's controller.
pub fn in_your_hand(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    q.obj(card).zone == Zone::hand(asker.controller)
}
