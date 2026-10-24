//! Day and night (CR 731), and the daybound and nightbound keyword abilities over them (CR 702.145).
//!
//! The designation belongs to the game, so everything about it is the rules' and nothing is a card's: the keywords
//! only say which of the two abilities a face is. It changes in exactly three places, each calling in here: the untap
//! step's own turn-based action (`check_untap_step`, CR 502.2, 731.2), a board of permanents with daybound or
//! nightbound finding a game that is neither (`apply`, CR 702.145d and 702.145g), and an effect saying so outright
//! (`become`, CR 731.1). A change always turns over whatever it has to before it returns, which is what makes
//! CR 702.145c's "this happens immediately" exact.

use cantrip_api::*;

use crate::R;
use crate::layers::live;
use crate::{Cx, ops};

/// The faces that have daybound or nightbound on the battlefield, as the state-based action pass found them.
pub struct Bound {
    pub id: ObjectId,
    pub daybound: bool,
}

/// Which of the two abilities a set of keywords has, if either.
pub fn role(keywords: Keywords) -> Option<bool> {
    if keywords.has(Keyword::Daybound) {
        Some(true)
    } else if keywords.has(Keyword::Nightbound) {
        Some(false)
    } else {
        None
    }
}

/// CR 702.145d and 702.145g, then CR 702.145c and 702.145f: a game that is neither day nor night gains a designation
/// from any permanent with either keyword, and every permanent whose face disagrees with the designation is turned
/// over. Whether anything happened.
pub fn apply(cx: &mut Cx, bound: &[Bound]) -> R<bool> {
    let mut acted = false;
    // Daybound is asked first: nightbound makes it night only while no permanent with daybound is out.
    if cx.s().turn.day_night == DayNight::Neither && !bound.is_empty() {
        let gained = if bound.iter().any(|b| b.daybound) { DayNight::Day } else { DayNight::Night };
        cx.st().update_turn(|t| t.day_night = gained);
        acted = true;
    }
    Ok(turn_disagreeing(cx, bound)? || acted)
}

/// CR 731.1: makes it day or night because something said so. Everything whose face now disagrees is turned over
/// before this returns. Nothing happens when the game already has that designation; nothing here ever takes one away.
pub fn become_designation(cx: &mut Cx, designation: DayNight) -> R<()> {
    if designation == DayNight::Neither || cx.s().turn.day_night == designation {
        return Ok(());
    }
    cx.st().update_turn(|t| t.day_night = designation);
    let bound = permanents(cx);
    turn_disagreeing(cx, &bound)?;
    Ok(())
}

/// The untap step's second turn-based action (CR 502.2, 731.2): if it is day and the previous turn's active player
/// cast no spells during that turn, it becomes night; if it is night and they cast two or more, it becomes day. A game
/// that is neither is not checked at all.
pub fn check_untap_step(cx: &mut Cx) -> R<()> {
    let turn = cx.s().turn;
    if turn.day_night == DayNight::Neither || turn.number < 2 {
        return Ok(());
    }
    // CR 726.2: what the previous turn's active player cast during that turn.
    let previous = turn.number - 1;
    let cast = cx.history().all().iter().filter(|r| r.turn == previous && matches!(r.event, Event::Cast(c) if c.player == turn.previous)).count();
    let next = match turn.day_night {
        DayNight::Day if cast == 0 => DayNight::Night,
        DayNight::Night if cast >= 2 => DayNight::Day,
        _ => return Ok(()),
    };
    become_designation(cx, next)
}

/// CR 702.145b's first static ability: "if it is night and this permanent is represented by a double-faced card, it
/// enters transformed", asked of a card on its way to the battlefield from wherever it is coming.
/// Asked first of its printed daybound; one entering as a copy is judged again as that copy once the copy is chosen
/// (`ops::prepare_entry`).
pub fn enters_transformed(cx: &Cx, card: ObjectId) -> bool {
    night_and_double_faced(cx, card) && cx.card(cx.s().obj(card).card).keywords.has(Keyword::Daybound)
}

/// Whether it is night and `card` is represented by a double-faced card (CR 702.145b), so that daybound it would have
/// makes it enter transformed.
pub fn night_and_double_faced(cx: &Cx, card: ObjectId) -> bool {
    cx.s().turn.day_night == DayNight::Night && double_faced(cx, card)
}

/// Whether `id` is represented by a double-faced card or token (CR 701.27a, 707.8a), which alone can be turned over.
fn double_faced(cx: &Cx, id: ObjectId) -> bool {
    let obj = cx.s().obj(id);
    obj.represented_double_faced(cx.cards().backs[obj.card.0 as usize].is_some())
}

/// Every permanent with daybound or nightbound on the battlefield.
fn permanents(cx: &Cx) -> Vec<Bound> {
    cx.s()
        .battlefield()
        .iter()
        .filter_map(|&id| role(cx.chars(id).keywords).map(|daybound| Bound { id, daybound }))
        .collect()
}

/// CR 702.145c and 702.145f: any time a permanent is front face up with daybound and it is night, or back face up
/// with nightbound and it is day, it is turned over. One at a time and read again between each, because turning a
/// permanent over changes which of the two keywords it has. Which face is up is the permanent's, not its keyword's: a
/// copy of a daybound creature that entered with its back face up (CR 702.145b) keeps daybound and is not turned over,
/// and one that can't be turned over (a back face that is a sorcery, CR 712.10) is asked no more.
fn turn_disagreeing(cx: &mut Cx, bound: &[Bound]) -> R<bool> {
    let mut acted = false;
    let mut bound: Vec<Bound> = bound.iter().map(|b| Bound { id: b.id, daybound: b.daybound }).collect();
    let mut stuck: Vec<ObjectId> = Vec::new();
    // A rules bug should surface as a clear stop rather than a hung game.
    for _ in 0..100 {
        let designation = cx.s().turn.day_night;
        let Some(next) = bound.iter().find(|b| {
            let obj = cx.s().obj(b.id);
            live(cx.s(), b.id)
                && obj.zone.is(ZoneKind::Battlefield)
                && double_faced(cx, b.id)
                && !stuck.contains(&b.id)
                && ((b.daybound && !obj.transformed && designation == DayNight::Night)
                    || (!b.daybound && obj.transformed && designation == DayNight::Day))
        }) else {
            return Ok(acted);
        };
        let id = next.id;
        if ops::turn_over(cx, id)? {
            acted = true;
            bound = permanents(cx);
        } else {
            stuck.push(id);
        }
    }
    Ok(acted)
}
