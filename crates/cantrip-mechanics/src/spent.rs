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

/// Opus (an ability word, CR 207.2c): "if five or more mana was spent to cast that spell", where the spell is the one the
/// ability triggered on (`item.subject`). Its mana spent (CR 601.2h) is read from the spell's own object, so it still answers once
/// the spell has been countered or has left the stack (CR 608.2h); a spell cast without paying its mana cost has had none spent.
/// Colorstorm Stallion and Molten-Core Maestro print it.
pub fn five_or_more_mana_spent_on_the_spell(q: &dyn Query, item: &StackItem) -> bool {
    matches!(item.subject, Target::Object(spell) if q.obj(spell).spent.total() >= 5)
}
