//! Discover N (CR 701.57a): "Exile cards from the top of your library until you exile a nonland card with mana value N or
//! less. You may cast that card without paying its mana cost if the resulting spell's mana value is less than or equal to
//! N. If you don't cast it, put that card into your hand. Put the remaining exiled cards on the bottom of your library in a
//! random order." Quintorius Kand's second loyalty ability prints it (Geological Appraiser and Trumpeting Carnosaur, not in
//! content yet, are the other printed cards).
//!
//! A keyword action, so this is its one module: a card calls `discover(g, player, n)`.

use crate::{choose, library};
use cantrip_api::*;

/// "You may cast that card without paying its mana cost if the resulting spell's mana value is less than or equal to `n`.
/// If you don't cast it, put that card into your hand" (CR 701.57a, 601.2b), for the `card` just exiled: the player is asked
/// only when a face of it can be cast at that mana value, and the card goes to the hand when not cast. `discover` and Bre of
/// Clan Stoutarm (`n` the life gained this turn) print it.
pub fn cast_free_if_at_most_else_hand(g: &mut dyn Game, player: PlayerId, card: ObjectId, n: u32) -> Result<(), Suspend> {
    if !may_cast_free(g, player, card, n)? {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

/// "You may cast that card without paying its mana cost if the resulting spell's mana value is `most` or less" (CR 608.2g,
/// 118.9, 601.2b), as the effect resolves: the player is asked only when a face of `card` can be cast (at that mana value),
/// picks the face when it has several, and casts it for {0}. Whether it was cast; a cast given up (CR 733.1)
/// leaves the card where it was. Asked as "cast or put into hand" (`CastFoundOrHand`): the card goes to the hand when not cast. Discover, Bre of Clan Stoutarm and Beseech the Mirror print it.
pub fn may_cast_free(g: &mut dyn Game, player: PlayerId, card: ObjectId, most: u32) -> Result<bool, Suspend> {
    choose::may_cast_free_card(g, player, card, card, most, Purpose::CastFoundOrHand)
}

/// Discover `n` for `player`, as an effect resolves (CR 701.57a). The card exiled is cast without paying its
/// mana cost from exile (so it is a spell cast from exile, CR 601.2a) if the player says so and can: it needs a legal
/// target for every target it has, and the face it is cast as (a split card's half, an Adventurer's, a modal double-faced
/// card's) must itself have mana value `n` or less. A card not cast, whether declined, impossible or given up (CR 733.1),
/// goes into the player's hand; the other exiled cards go to the bottom of the library in a random order. Nothing is cast or
/// put into hand if the library ran out first (CR 701.57b: it has still been discovered).
pub fn discover(g: &mut dyn Game, player: PlayerId, n: u32) -> Result<(), Suspend> {
    let (exiled, hit) = library::exile_until_nonland(g, player, None, &|q, card| q.mana_value(card) <= n);
    if let Some(card) = hit {
        cast_free_if_at_most_else_hand(g, player, card, n)?;
    }
    // What was cast or put into hand has left exile with its identity; the rest go to the bottom, shuffled among themselves.
    let rest: Vec<ObjectId> = exiled.into_iter().filter(|&c| is_live(g, c) && g.obj(c).zone.is(ZoneKind::Exile)).collect();
    g.put_into_library(&rest, LibraryPlace::BottomInRandomOrder);
    Ok(())
}
