//! Changing an object's text (CR 612): the words a text-changing spell replaces, chosen as it resolves (CR 608.2d), and
//! the effect that changes them (`EffectKind::ChangeText`, layer 3).
//!
//! "Replacing all instances of one color word with another or one basic land type with another" is printed on Mind Bend
//! and Trait Doctoring; a card that prints it sets its `labels` to `LABELS`, the two kinds of word it offers.

use cantrip_api::*;

/// What a "color word or basic land type" choice offers first, as the card's `labels`.
pub const LABELS: &[&str] = &["Replace a color word", "Replace a basic land type"];

const BASIC_LAND_TYPES: [Subtype; 5] = [Subtype::Plains, Subtype::Island, Subtype::Swamp, Subtype::Mountain, Subtype::Forest];

/// One color word to be replaced with another, or one basic land type with another (CR 612.2), chosen by `player`: the
/// kind of word, the word, then the word it becomes, which is a different one.
pub fn color_word_or_basic_land_type(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<(Word, Word), Suspend> {
    let card = g.chars(source).card;
    let kind = crate::choose::one_of(g, player, source, vec![Label::Card(card, 0), Label::Card(card, 1)])?;
    if kind == 0 {
        let from = one_color(g, player, source, ColorSet(0b11111))?;
        let to = one_color(g, player, source, ColorSet(0b11111 & !ColorSet::of(from).0))?;
        Ok((Word::Color(from), Word::Color(to)))
    } else {
        let from = one_subtype(g, player, source, BASIC_LAND_TYPES.to_vec())?;
        let to = one_subtype(g, player, source, BASIC_LAND_TYPES.into_iter().filter(|&s| s != from).collect())?;
        Ok((Word::Subtype(from), Word::Subtype(to)))
    }
}

/// One color of `choices`.
fn one_color(g: &mut dyn Game, player: PlayerId, source: ObjectId, choices: ColorSet) -> Result<Color, Suspend> {
    match g.ask(player, RequestKind::ChooseColors { source, choices, count: 1 })? {
        Answer::Colors(colors) => Ok(colors[0]),
        _ => unreachable!("the core answers a color choice with colors"),
    }
}

/// One subtype of `choices`.
pub fn one_subtype(g: &mut dyn Game, player: PlayerId, source: ObjectId, choices: Vec<Subtype>) -> Result<Subtype, Suspend> {
    match g.ask(player, RequestKind::ChooseSubtype { source, choices })? {
        Answer::Subtype(s) => Ok(s),
        _ => unreachable!("the core answers a subtype choice with a subtype"),
    }
}

/// "Change the text of [it] by replacing all instances of `from` with `to`" for `duration` (CR 612.1, 611.2): an object
/// that has left the stack or the battlefield since it was targeted is a new object, and nothing changes.
pub fn change(g: &mut dyn Game, source: ObjectId, object: ObjectId, (from, to): (Word, Word), duration: Duration) {
    let obj = g.obj(object);
    if is_live(g, object) && (obj.zone.is(ZoneKind::Battlefield) || obj.zone.is(ZoneKind::Stack)) {
        g.add_effect(source, object, EffectKind::ChangeText { from, to }, duration);
    }
}
