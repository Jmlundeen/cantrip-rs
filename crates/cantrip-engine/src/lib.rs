//! The engine for its users (the scenario runner, the benchmarks, later the host and the Python bindings): the rules,
//! and the card table every card crate contributes to, built once per process and never copied.

use std::sync::OnceLock;

pub mod undo;

pub use cantrip_api::*;
pub use cantrip_rules::{Game, Rejected, Setup, chars, check_answer, describe, distribution_problem, live, name_problem, summoning_sick};

/// The simplest legal distribution of looked-at cards (the C# `CardDistribution.Greedy`): each destination's
/// minimum first, in order, then each takes what it accepts up to its maximum, cards in the order looked at.
pub fn greedy_distribution(cards: &[ObjectId], destinations: &[Destination]) -> Vec<Vec<ObjectId>> {
    let mut left: Vec<ObjectId> = cards.to_vec();
    let mut placements: Vec<Vec<ObjectId>> = vec![Vec::new(); destinations.len()];
    for pass in 0..2 {
        for (i, d) in destinations.iter().enumerate() {
            let up_to = if pass == 0 { d.min as usize } else if d.max == u8::MAX { usize::MAX } else { d.max as usize };
            let mut j = 0;
            while j < left.len() && placements[i].len() < up_to {
                if d.eligible.contains(&left[j]) {
                    placements[i].push(left.remove(j));
                } else {
                    j += 1;
                }
            }
        }
    }
    placements
}
pub use cantrip_state::{Frame, FrameKind, Mark};

/// Every card, sorted by name.
pub fn cards() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all()))
}

/// The card a name names, ignoring case.
pub fn card(name: &str) -> Option<CardId> {
    cards().find(name)
}

/// A game from decklists of names, top of the library first.
pub fn deal(decks: &[Vec<String>], seed: u64, shuffle: bool, opening_chooser: Option<PlayerId>) -> Result<Game, String> {
    let mut ids = Vec::new();
    for deck in decks {
        let mut list = Vec::new();
        for name in deck {
            // Either face of a double-faced card names it (CR 711.1); a deck holds the front.
            let id = card(name).ok_or_else(|| format!("no card is named \"{name}\""))?;
            list.push(cards().fronts[id.0 as usize].unwrap_or(id));
        }
        ids.push(list);
    }
    let mut game = Game::new(cards(), &Setup { decks: ids, seed, shuffle, opening_chooser });
    game.run();
    Ok(game)
}

/// A label's text, as a client shows it and a scenario names it.
pub fn label_text(label: &Label) -> &'static str {
    match *label {
        Label::Card(card, i) => cards().get(card).labels.get(i as usize).copied().unwrap_or(""),
        Label::Name(card) | Label::Face(card) => cards().get(card).name,
        Label::Pile { first: true, empty: true } => "First pile: nothing",
        Label::Pile { first: true, empty: false } => "First pile",
        Label::Pile { first: false, empty: true } => "Second pile: nothing",
        Label::Pile { first: false, empty: false } => "Second pile",
        // A modal spell's or ability's mode, from the card's own definition (its spell's, or its first modal ability's).
        Label::Mode(ability, i) => {
            let def = cards().get(ability.card);
            let modal = if ability.index == u8::MAX {
                def.modal
            } else {
                match def.abilities.get(ability.index as usize) {
                    Some(AbilityDef::Activated(a)) => a.modal,
                    Some(AbilityDef::Triggered(t)) => t.modal,
                    _ => None,
                }
            };
            modal.and_then(|m| m.modes.get(i as usize)).map_or("", |m| m.text)
        }
        Label::Extra(card, i) => cards().get(card).additional_choice.get(i as usize).map_or("", |c| c.text),
        Label::PayLife(Color::White) => "Pay 2 life for {W/P}",
        Label::PayLife(Color::Blue) => "Pay 2 life for {U/P}",
        Label::PayLife(Color::Black) => "Pay 2 life for {B/P}",
        Label::PayLife(Color::Red) => "Pay 2 life for {R/P}",
        Label::PayLife(Color::Green) => "Pay 2 life for {G/P}",
        Label::Counter(kind) => kind.name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A creature whose power or toughness is printed as a star has it from a characteristic-defining ability (CR 208.2a), and
    /// once that is lost (CR 604.3, 613.1f) the star is worth 0 (CR 208.5): `StaticDef::lost_pt` says what it leaves, so the next
    /// card with a star cannot forget it.
    #[test]
    fn every_star_creature_says_what_its_power_and_toughness_are_once_the_ability_is_lost() {
        for def in &cards().cards {
            if !def.is(CardType::Creature) || (def.power.is_some() && def.toughness.is_some()) {
                continue;
            }
            for st in def.statics.iter().filter(|s| s.own && s.layer == Layer::PtDefine) {
                assert!(st.lost_pt.is_some(), "{}: its power and toughness are defined by an ability, and `.lost_pt(power, toughness)` is missing", def.name);
            }
        }
    }
}
