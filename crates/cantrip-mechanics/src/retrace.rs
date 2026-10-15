//! Retrace (CR 702.81a): "You may cast this card from your graveyard by discarding a land card as an additional cost to
//! cast it." The keyword is the core's (`Keyword::Retrace`); Six and Wrenn and Six's emblem give it to the cards in their
//! controller's graveyard.
//!
//! A card that grants retrace lists a static whose `apply` is `give`.

use cantrip_api::*;

/// The affected cards have retrace (layer 6).
pub fn give(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Retrace);
}
