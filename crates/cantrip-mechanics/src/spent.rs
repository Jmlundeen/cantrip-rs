//! "If {C}{C} was spent to cast it": the intervening "if" of the Elemental Incarnations of Modern Horizons 3 (Vibrance,
//! Wistfulness, Deceit). The mana is counted over the whole payment of the spell (CR 601.2h, 106.1b): mana of that color
//! spent on generic costs and on an evoke cost counts too, and a third mana of the color changes nothing. A permanent that
//! was not cast (put onto the battlefield by an effect, or a copy of a spell, CR 707.10) had nothing spent, and a permanent
//! that copies one has what it was cast with, not what it copies (CR 707.2).

use cantrip_api::*;

/// Whether at least two mana of the kind `M` (a `Mana` as `u8`: `spent::two::<{ Mana::U as u8 }>`) were spent to cast the
/// source of the ability. Read from the object the ability came from, so it still answers when that has left the
/// battlefield (CR 113.7a).
pub fn two<const M: u8>(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).spent.get(Mana::ALL[M as usize]) >= 2
}
