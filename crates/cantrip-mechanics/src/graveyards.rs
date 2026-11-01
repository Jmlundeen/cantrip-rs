//! "A card in a graveyard", any graveyard (CR 404.1): every player's graveyard is read, in seat order and each in the order
//! the zone keeps it (the order its cards arrived, the top last).

use cantrip_api::*;

/// Every card in every player's graveyard: Breaking // Entering's "a creature card from a graveyard", Superior Spider-Man's "any
/// creature card in a graveyard", Echoing Deeps's "a land card in a graveyard" and Necrotic Ooze's "creature cards in all
/// graveyards". The caller narrows them by card type.
pub fn cards_in_graveyards(s: &GameState) -> impl Iterator<Item = ObjectId> + '_ {
    s.seats().flat_map(|p| s.zone(Zone::graveyard(p)).iter().copied())
}
