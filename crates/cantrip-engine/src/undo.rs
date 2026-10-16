//! Taking back a mana ability: a courtesy of the table, not a rule, so it is offered only while nothing hidden has been
//! shown and nothing has followed from it. While the last answer a player gave activated a mana ability (CR 605.1a),
//! and it and everything since only tapped, sacrificed from the battlefield, removed counters, added mana or cost life
//! - a land, Black Lotus, Lotus Petal, Pentad Prism, a Treasure - the player may take it back, and the game goes back to
//! before it through the game-log tree (`Game::goto`). Never taken back: an ability whose cost discards (Lion's Eye
//! Diamond shows the hand), and one that triggered anything (Chromatic Star draws). Any other answer, casting a spell
//! included, clears what can be taken back.
//!
//! The host and the scenario runner share it: what one offers, the other can script (`Bot1.Undo`).

use cantrip_api::*;
use cantrip_rules::Game;
use cantrip_rules::tree::NodeId;

/// The mana abilities that can still be taken back, latest last, each with what its keeper holds beside it (the host's
/// told events and handles; nothing for the scenario runner).
#[derive(Clone)]
pub struct Undo<T> {
    entries: Vec<Entry<T>>,
}

#[derive(Clone)]
struct Entry<T> {
    player: PlayerId,
    source: ObjectId,
    node: NodeId,
    kept: T,
}

/// An answer about to be given, as taking it back needs it remembered.
pub struct Before {
    player: PlayerId,
    source: Option<ObjectId>,
    node: NodeId,
    log_from: usize,
    triggers: usize,
    stack: usize,
}

impl Before {
    /// Whether the answer activates a mana ability that could be taken back, for a keeper to know whether to save
    /// what it would need.
    pub fn is_mana(&self) -> bool {
        self.source.is_some()
    }
}

impl<T> Default for Undo<T> {
    fn default() -> Self {
        Undo { entries: Vec::new() }
    }
}

impl<T> Undo<T> {
    /// Notes `answer`, about to be given by `player`.
    pub fn before(game: &Game, player: PlayerId, answer: &Answer) -> Before {
        Before {
            player,
            source: mana_activation(game, answer),
            node: game.checkpoint(),
            log_from: game.log.len(),
            triggers: game.state.triggers.len(),
            stack: game.state.stack.len(),
        }
    }

    /// Whether the answer `before` noted, now given, may be taken back. If not, nothing before it may be either.
    pub fn may_take_back(&mut self, game: &Game, before: &Before) -> bool {
        let asks_again = game.pending().is_some_and(|next| {
            next.player == before.player && matches!(next.kind, RequestKind::Priority { .. } | RequestKind::PayCosts { .. })
        });
        // A trigger is something the ability did (Chromatic Star's draw), waiting or on the stack, even before anyone
        // has seen what it does.
        let undoable = before.source.is_some()
            && asks_again
            && only_mana(&game.log[before.log_from.min(game.log.len())..])
            && game.state.triggers.len() <= before.triggers
            && game.state.stack.len() <= before.stack;
        if !undoable {
            self.entries.clear();
        }
        undoable
    }

    /// Records the answer `before` noted as one that may be taken back, with what the keeper holds beside it. Call
    /// only when [`Undo::may_take_back`] said so.
    pub fn push(&mut self, before: Before, kept: T) {
        let source = before.source.expect("a mana ability");
        self.entries.push(Entry { player: before.player, source, node: before.node, kept });
    }

    /// The permanent whose mana ability `player` may take back now, if any.
    pub fn offered(&self, player: PlayerId) -> Option<ObjectId> {
        self.entries.last().filter(|e| e.player == player).map(|e| e.source)
    }

    /// What the keeper holds beside the latest entry.
    pub fn last_kept(&mut self) -> Option<&mut T> {
        self.entries.last_mut().map(|e| &mut e.kept)
    }

    /// Takes back the latest mana ability `player` activated: the game goes back to the node before it, and what the
    /// keeper held beside it is returned. None if `player` has nothing to take back.
    pub fn take_back(&mut self, game: &mut Game, player: PlayerId) -> Option<T> {
        self.offered(player)?;
        let entry = self.entries.pop()?;
        game.goto(entry.node);
        Some(entry.kept)
    }

    /// Nothing may be taken back any more: a concession, or the game set aside.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// The source of a mana ability `answer` activates (CR 605.1a), if it activates one that could be taken back: not one
/// whose cost discards, which shows what was in the hand.
fn mana_activation(game: &Game, answer: &Answer) -> Option<ObjectId> {
    let Answer::Act(action) = answer else { return None };
    let source = action.source.filter(|_| action.kind == ActionKind::ActivateAbility)?;
    if action.index >= PLAYER_MANA_ABILITY {
        return Some(source);
    }
    match game.chars(source).abilities().nth(action.index as usize).map(|a| game.cards.ability(a)) {
        Some(AbilityDef::Activated(d)) if d.is_mana_ability() && !d.cost.discard_hand && !d.cost.discard_self && d.cost.discard.is_none() => {
            Some(source)
        }
        _ => None,
    }
}

/// Whether these entries are only what a mana ability does: taps, sacrifices from the battlefield, mana, and life paid
/// or lost for it.
fn only_mana(records: &[Record]) -> bool {
    records.iter().all(|r| match r.event {
        Event::ZoneChange { from, reason, .. } => reason == ZoneReason::Sacrifice && from.kind() == ZoneKind::Battlefield,
        Event::Tapped { .. }
        | Event::TappedForMana { .. }
        | Event::LifeChange { .. }
        | Event::AbilityActivated { .. }
        | Event::Resolved { .. }
        | Event::Damage { target: Target::Player(_), .. } => true,
        _ => false,
    })
}
