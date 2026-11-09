//! The Scarab cycle's Aura text: "Enchanted creature can't be blocked by [color] creatures. Enchanted creature gets +2/+2 as long
//! as an opponent controls a [color] permanent." A card lists `cant_be_blocked_by_color::<COLOR>` in its `block_rules` and
//! `enchanted_gets_while_opponent_controls_color::<COLOR>` as the `affects` of a `Layer::PtModify` static with
//! `statics::plus_pt::<2, 2>`; `COLOR` is `scarab::color(Color::Black)`. The color is read through the words of the Aura's text
//! (CR 612). Black Scarab and Blue Scarab print it (Green, Red and White Scarab too).

use cantrip_api::*;

/// The const argument of the Scarab recipes.
pub const fn color(color: Color) -> u8 {
    color as u8
}

/// "Enchanted creature can't be blocked by [color] creatures" (CR 509.1b) as a `CardDef::block_rules` entry: a rule of the Aura's,
/// not an ability the creature gains (CR 113.1a).
pub fn cant_be_blocked_by_color<const COLOR: u8>(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    q.obj(source).attached_to != Some(attacker)
        || !(q.chars(blocker).is(CardType::Creature) && q.chars(blocker).colors.has(q.words(source).color(Color::ALL[COLOR as usize])))
}

/// The `affects` of "Enchanted creature gets +2/+2 as long as an opponent controls a [color] permanent" (CR 604.1, 613.4c): the
/// creature the Aura is attached to, while some opponent of the Aura's controller controls a permanent of that color.
pub fn enchanted_gets_while_opponent_controls_color<const COLOR: u8>(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let color = q.words(source).color(Color::ALL[COLOR as usize]);
    q.state().obj(source).attached_to == Some(affected)
        && q.state().opponents(q.controller(source)).any(|opponent| {
            q.state().battlefield().iter().any(|&id| q.controller(id) == opponent && q.chars(id).colors.has(color))
        })
}
