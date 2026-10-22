//! Selection rules (docs/engine-design.md, "Selection rules"): which sets of cards answer a choice of cards or a library
//! search. A rule (`Selection`) is plain data in the pending question - a count, and parts that are each a plain function
//! (`SelectFn`) with the one number it takes - and `judge` runs it over the cards chosen so far: whether they are an answer
//! (`Verdict::complete`) and which further cards may be added to them (`Verdict::addable`). The core checks every answer
//! with it, a client asks it what may be picked next (the host's `Check`), and the scenario harness answers through it.
//!
//! The ready-made rules and parts new cards use are `cantrip_mechanics::select`; a card with a rule of its own writes a
//! `SelectFn` and makes it a part with `part`.

pub use cantrip_state::{Count, Part, Selection, Tally, MAX_PARTS};

use crate::Query;
use cantrip_state::{ObjectId, PlayerId};

/// A question's cards, and the ones chosen so far, as a part of its rule judges them.
#[derive(Clone, Copy, Debug)]
pub struct Choice<'a> {
    /// The player choosing.
    pub chooser: PlayerId,
    /// The object whose text asks the question.
    pub source: ObjectId,
    /// Every card of the question, in its order: what `Verdict::addable` is indexed by.
    pub candidates: &'a [ObjectId],
    /// The cards chosen so far, each one of `candidates`.
    pub selected: &'a [ObjectId],
}

/// A set of a question's candidates, by their places in its order: a bit for each.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bits {
    words: Vec<u64>,
    len: usize,
}

impl Bits {
    /// `len` places, every one of them `on` or none.
    pub fn new(len: usize, on: bool) -> Bits {
        let mut words = vec![if on { u64::MAX } else { 0 }; len.div_ceil(64)];
        if on && len % 64 != 0 {
            if let Some(last) = words.last_mut() {
                *last = (1u64 << (len % 64)) - 1;
            }
        }
        Bits { words, len }
    }

    /// The places `keep` is true of.
    pub fn of(len: usize, mut keep: impl FnMut(usize) -> bool) -> Bits {
        let mut bits = Bits::new(len, false);
        for i in 0..len {
            if keep(i) {
                bits.set(i, true);
            }
        }
        bits
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, i: usize) -> bool {
        i < self.len && self.words[i / 64] & (1 << (i % 64)) != 0
    }

    pub fn set(&mut self, i: usize, on: bool) {
        if i < self.len {
            if on {
                self.words[i / 64] |= 1 << (i % 64);
            } else {
                self.words[i / 64] &= !(1 << (i % 64));
            }
        }
    }

    /// Keeps only the places `other` has too.
    pub fn and(&mut self, other: &Bits) {
        for (i, word) in self.words.iter_mut().enumerate() {
            *word &= other.words.get(i).copied().unwrap_or(0);
        }
    }

    /// Whether any place is on.
    pub fn any(&self) -> bool {
        self.words.iter().any(|&w| w != 0)
    }

    /// The places that are on, in order.
    pub fn ones(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.len).filter(|&i| self.get(i))
    }
}

/// What a rule, or one part of it, says of the cards chosen so far.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Verdict {
    /// The cards chosen are an answer: as many as it takes, and each and all of them as it asks.
    pub complete: bool,
    /// The candidates not chosen that may be added to them, by their places among the question's cards. Taking a card back
    /// out is always allowed.
    pub addable: Bits,
}

impl Verdict {
    /// Every candidate addable and the selection complete: what a part that asks nothing says.
    pub fn any(choice: &Choice) -> Verdict {
        Verdict { complete: true, addable: Bits::new(choice.candidates.len(), true) }
    }

    /// A quality each card must have, alone (CR 701.23b's "stated quality"): a candidate is addable when it has it, and the
    /// selection is complete when every card chosen has it.
    pub fn each(choice: &Choice, mut has: impl FnMut(ObjectId) -> bool) -> Verdict {
        let addable = Bits::of(choice.candidates.len(), |i| has(choice.candidates[i]));
        let complete = choice.selected.iter().all(|&c| choice.candidates.iter().position(|&x| x == c).is_some_and(|i| addable.get(i)));
        Verdict { complete, addable }
    }

    /// The candidates addable to it, as objects, in the question's order.
    pub fn addable_cards(&self, choice: &Choice) -> Vec<ObjectId> {
        self.addable.ones().filter_map(|i| choice.candidates.get(i).copied()).collect()
    }
}

/// A part's validator: what it says of `choice` (the question's cards and those chosen so far), given its number. Read
/// the game through `q` and nothing else, so that the same position always gets the same verdict.
pub type SelectFn = fn(q: &dyn Query, choice: &Choice, n: u32) -> Verdict;

/// A part made of `judge`, with its number and its words (`Part::words`). Called with the function as a constant
/// reference, `part(&(my_rule as SelectFn), 0, "with ...")`, which the compiler keeps for the whole program.
pub const fn part(judge: &'static SelectFn, n: u32, words: &'static str) -> Part {
    Part { judge, n, words, tally: Tally::Nothing }
}

/// `part`, with what a client keeps count of for the player as the cards are chosen (`Tally`).
pub const fn tallied(judge: &'static SelectFn, n: u32, words: &'static str, tally: Tally) -> Part {
    Part { judge, n, words, tally }
}

/// The validator a part holds.
fn validator(part: &Part) -> SelectFn {
    *part.judge.downcast_ref::<SelectFn>().expect("a selection rule's part holds a SelectFn (cantrip_api::select::part)")
}

/// What `rule` says of `choice`: every part's verdict together, then the count. A card is addable when it is a candidate
/// not yet chosen, `among` lists it, every part allows it and the count has room; the cards chosen are complete when they
/// are distinct candidates `among` lists, every part calls them complete, and they are as many as the count takes - for
/// `Count::AsManyAsCan`, fewer only when nothing could be added.
pub fn judge(q: &dyn Query, rule: &Selection, choice: &Choice) -> Verdict {
    let candidates = choice.candidates;
    let chosen = |c: ObjectId| choice.selected.contains(&c);
    let mut addable = Bits::of(candidates.len(), |i| !chosen(candidates[i]) && rule.listed_card(candidates[i]));
    let mut complete = choice.selected.iter().enumerate().all(|(i, c)| candidates.contains(c) && rule.listed_card(*c) && !choice.selected[..i].contains(c));
    for part in rule.parts() {
        let verdict = validator(part)(q, choice, part.n);
        addable.and(&verdict.addable);
        complete &= verdict.complete;
    }
    let more = addable.any();
    complete &= rule.count.complete(choice.selected.len(), more);
    if !rule.count.room(choice.selected.len()) {
        addable = Bits::new(candidates.len(), false);
    }
    Verdict { complete, addable }
}

/// Whether `selected` answers a question of `candidates` with `rule`, asked of `chooser` by `source`.
pub fn answers(q: &dyn Query, rule: &Selection, chooser: PlayerId, source: ObjectId, candidates: &[ObjectId], selected: &[ObjectId]) -> bool {
    judge(q, rule, &Choice { chooser, source, candidates, selected }).complete
}

/// An answer reached by adding the first addable card again and again: with `most`, until the count is reached or nothing
/// more may be added (the most the rule allows, in the question's order: the answer of a player who takes the first cards
/// offered); without it, only until the cards are complete (none at all when none is an answer). `None` when adding cards
/// that way reaches no answer.
pub fn first_answer(q: &dyn Query, rule: &Selection, chooser: PlayerId, source: ObjectId, candidates: &[ObjectId], most: bool) -> Option<Vec<ObjectId>> {
    let mut selected: Vec<ObjectId> = Vec::new();
    loop {
        let verdict = judge(q, rule, &Choice { chooser, source, candidates, selected: &selected });
        let next = verdict.addable.ones().next();
        match next {
            Some(i) if most || !verdict.complete => selected.push(candidates[i]),
            _ => return verdict.complete.then_some(selected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bit set over more candidates than one word holds keeps every place apart, and "all on" covers exactly its length.
    #[test]
    fn bits_hold_places_past_one_word() {
        let mut bits = Bits::new(70, true);
        assert_eq!(bits.ones().count(), 70);
        bits.set(65, false);
        assert!(!bits.get(65) && bits.get(64) && bits.get(69) && !bits.get(70));
        let mut some = Bits::of(70, |i| i % 2 == 0);
        some.and(&bits);
        assert_eq!(some.ones().count(), 35, "every even place of 70, the one turned off (65) being odd");
        assert!(!Bits::new(3, false).any());
    }

    /// What each count calls complete, and when it has room for one more.
    #[test]
    fn counts_say_how_many_complete_a_selection() {
        assert!(Count::Exactly(2).complete(2, true) && !Count::Exactly(2).complete(1, false));
        assert!(Count::UpTo(2).complete(0, true) && !Count::UpTo(2).room(2));
        assert!(Count::AnyNumber.complete(0, true) && Count::AnyNumber.room(usize::MAX - 1));
        // As many as can be: short of the count only when nothing more could be added.
        assert!(!Count::AsManyAsCan(2).complete(1, true) && Count::AsManyAsCan(2).complete(1, false));
        assert!(!Count::AsManyAsCan(1).complete(0, true) && Count::AsManyAsCan(1).complete(0, false));
    }
}
