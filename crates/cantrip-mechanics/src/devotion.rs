//! Devotion (CR 700.5), a count cards read: Thassa's Oracle, Nykthos, Shrine to Nyx, Setessan Petitioner.
use cantrip_api::*;

/// Mana symbols of `color` in a permanent's or spell's mana cost (CR 700.5, 202.2; Namor the Sub-Mariner counts a spell's): hybrid ({U/R}, {2/U}) and Phyrexian symbols of the
/// color each count once, numeric and other colors' symbols not. A copy counts the cost it copies (CR 707.2, 700.5a),
/// none if it has none (CR 707.9b); a Room permanent counts its unlocked doors alone (CR 709.5).
pub fn symbols_of(q: &dyn Query, id: ObjectId, color: Color) -> u32 {
    let mana = color.mana();
    // The mana cost the object has now: a copy's is the one it copies, none if it has none (CR 707.9b), and a Room
    // permanent's is its unlocked doors' alone (CR 709.5).
    let cost = q.mana_cost(id);
    cost.symbols[mana as usize]
        + cost.hybrid.iter().flatten().filter(|(a, b)| *a == mana || *b == mana).count() as u32
        + cost.phyrexian[mana as usize]
        + cost.twobrid[mana as usize]
}

/// "Your devotion to [color]" (CR 700.5): the mana symbols of `color` among the mana costs of permanents `player`
/// controls, counted now.
pub fn devotion(q: &dyn Query, player: PlayerId, color: Color) -> u32 {
    q.permanents_of(player).into_iter().map(|id| symbols_of(q, id, color)).sum()
}
