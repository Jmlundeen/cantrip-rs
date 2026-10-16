//! A player choosing as an effect resolves (CR 608.2d): a card, some cards, or a player, asked as the core's question and
//! answered as the thing chosen. The core checks every answer against its question before it is taken (a card that was
//! not offered, or the wrong number of them, is refused and asked again), so a card's file reads the choice and nothing
//! else; what it does with "nothing to choose" is its own.

use cantrip_api::*;

/// `count` of `eligible` (or up to `count` with `up_to`), chosen from among `shown`, every one of which the player sees:
/// Aesir Escape Valhalla shows the whole graveyard and lets only its permanent cards be chosen. Nothing is asked when
/// nothing is eligible. Fewer than `count` come back only when fewer were eligible, or when the choice is a discard or
/// sacrifice the player can't be made to make (CR 101.2).
pub fn cards_from(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    purpose: Purpose,
    shown: Vec<ObjectId>,
    eligible: Vec<ObjectId>,
    count: u8,
    up_to: bool,
) -> Result<Vec<ObjectId>, Suspend> {
    cards_from_kind(g, player, source, purpose, "", shown, eligible, count, up_to)
}

/// `cards_from` for a choice the asking card words with more than "card": `kind` is its own filter ("nonland", "artifact or
/// creature"), which the question says to the chooser. Elite Spellbinder and Tidehollow Sculler print "a nonland card".
pub fn cards_from_kind(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    purpose: Purpose,
    kind: &'static str,
    shown: Vec<ObjectId>,
    eligible: Vec<ObjectId>,
    count: u8,
    up_to: bool,
) -> Result<Vec<ObjectId>, Suspend> {
    if eligible.is_empty() || count == 0 {
        return Ok(Vec::new());
    }
    // "Any number" (`u8::MAX`, up to) stays the number the question was asked with, so it can say "any number of": a client
    // told "up to 2" of two cards could not tell it from "up to two".
    let count = if up_to && count == u8::MAX { count } else { count.min(eligible.len().min(u8::MAX as usize) as u8) };
    match g.ask(player, RequestKind::ChooseCards { source, purpose, cards: shown, eligible, count, up_to, kind })? {
        Answer::Cards(chosen) => Ok(chosen),
        _ => Ok(Vec::new()),
    }
}

/// `count` of `among` (or up to `count` with `up_to`), all of them shown: `cards_from` with nothing shown but what may be
/// chosen.
pub fn cards(g: &mut dyn Game, player: PlayerId, source: ObjectId, purpose: Purpose, among: Vec<ObjectId>, count: u8, up_to: bool) -> Result<Vec<ObjectId>, Suspend> {
    cards_from(g, player, source, purpose, among.clone(), among, count, up_to)
}

/// One of `among`, asked even when the choice is forced: none only when `among` is empty, or when the choice is a discard
/// or sacrifice the player can't be made to make (CR 101.2).
pub fn card(g: &mut dyn Game, player: PlayerId, source: ObjectId, purpose: Purpose, among: Vec<ObjectId>) -> Result<Option<ObjectId>, Suspend> {
    Ok(cards(g, player, source, purpose, among, 1, false)?.first().copied())
}

/// One of `eligible`, chosen from among `shown`: `card` for a choice made in view of cards that may not be chosen.
pub fn card_from(g: &mut dyn Game, player: PlayerId, source: ObjectId, purpose: Purpose, shown: Vec<ObjectId>, eligible: Vec<ObjectId>) -> Result<Option<ObjectId>, Suspend> {
    Ok(cards_from(g, player, source, purpose, shown, eligible, 1, false)?.first().copied())
}

/// A player of `candidates`, chosen without targeting (CR 608.2d, 614.12a), asked even when the choice is forced; none
/// only when there are no candidates. `verb` and `amount` say what the chosen player will do, as hint text.
pub fn player(g: &mut dyn Game, chooser: PlayerId, source: ObjectId, candidates: Vec<PlayerId>, verb: &'static str, amount: u16) -> Result<Option<PlayerId>, Suspend> {
    if candidates.is_empty() {
        return Ok(None);
    }
    match g.ask(chooser, RequestKind::ChoosePlayer { source, candidates, verb, amount })? {
        Answer::Player(p) => Ok(Some(p)),
        _ => Ok(None),
    }
}

/// One of the labelled options `modes`, by index (a pile, a mode, "tap or untap"), asked even when the choice is forced.
pub fn one_of(g: &mut dyn Game, player: PlayerId, source: ObjectId, modes: Vec<Label>) -> Result<u8, Suspend> {
    let eligible = (0..modes.len() as u8).collect();
    let ask = RequestKind::ChooseModes { source, modes, eligible, count: 1, at_most: 0, affordable: Vec::new(), additional_cost: false, repeat: false };
    match g.ask(player, ask)? {
        Answer::Modes(chosen) => Ok(chosen.first().copied().unwrap_or(0)),
        _ => Ok(0),
    }
}

/// The face a card is cast as (CR 709.3, 715.3, 712.8a), chosen from `ways` (`Game::cast_ways_during_resolution`, narrowed to
/// the ones the effect allows: a mana value, a type), as the player puts it on the stack. Nothing is asked when there is one;
/// `None` when there is none. Uldaros Theorix, Beseech the Mirror and Bilbo, Thief in the Night print a restriction on the spell.
pub fn cast_way(g: &mut dyn Game, player: PlayerId, card: ObjectId, ways: Vec<CastWay>) -> Result<Option<CastWay>, Suspend> {
    match ways.len() {
        0 => Ok(None),
        1 => Ok(Some(ways[0])),
        _ => {
            let faces = ways.iter().map(|w| Label::Face(w.face)).collect();
            Ok(Some(ways[one_of(g, player, card, faces)? as usize]))
        }
    }
}

/// "You may cast any number of spells from among [these cards] without paying their mana costs" (CR 608.2g, 118.9): the
/// controller is offered the castable ones among `cards` one at a time, in the order they like, until they decline or none
/// is castable. Each is cast as the effect resolves, so it takes its face (when it has several) and its targets as any cast
/// does; a cast given up leaves the card where it was, to be offered again. A card is castable only while it is still
/// where it was (`could_cast_during_resolution` asks), so one already cast is not offered.
/// Etali, Primal Conqueror and Improvisation Capstone print it.
pub fn cast_any_number_free(g: &mut dyn Game, player: PlayerId, source: ObjectId, cards: &[ObjectId]) -> Result<(), Suspend> {
    cast_up_to_free(g, player, source, cards, usize::MAX)
}

/// "You may cast up to `most` spells from among [these cards] without paying their mana costs": as `cast_any_number_free`,
/// nothing more offered once `most` have been cast; a cast given up does not count. Doom Reigns Supreme uses it with
/// two; Plargg and Nassari (not in content yet) is the second printed card with the same words.
pub fn cast_up_to_free(g: &mut dyn Game, player: PlayerId, source: ObjectId, cards: &[ObjectId], most: usize) -> Result<(), Suspend> {
    let mut cast = 0;
    while cast < most {
        let castable: Vec<ObjectId> = cards.iter().copied().filter(|&c| g.could_cast_during_resolution(c)).collect();
        let chosen = cards_from(g, player, source, Purpose::CastFree, cards.to_vec(), castable, 1, true)?;
        let Some(&card) = chosen.first() else { return Ok(()) };
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
        if g.obj(card).became.is_some() {
            cast += 1;
        }
    }
    Ok(())
}

/// The labels "choose odd or even" offers, odd first: a card that asks it lists them as its `labels`.
pub const ODD_OR_EVEN: &[&str] = &["Odd", "Even"];

/// "Choose odd or even" (Gollum, Riddle Master as it enters; Thanos, the Mad Titan's power-up): true for odd. `card` is the
/// card whose `labels` are `ODD_OR_EVEN`.
pub fn odd_or_even(g: &mut dyn Game, player: PlayerId, source: ObjectId, card: CardId) -> Result<bool, Suspend> {
    Ok(one_of(g, player, source, vec![Label::Card(card, 0), Label::Card(card, 1)])? == 0)
}

/// Whether an object's mana value is of the chosen quality, odd or even: zero is even (Gollum, Riddle Master; Thanos, the
/// Mad Titan).
pub fn mana_value_of_quality(q: &dyn Query, id: ObjectId, odd: bool) -> bool {
    (q.mana_value(id) % 2 == 1) == odd
}
