//! Replacement effects (CR 614, 616): a permanent's static ability that changes what an event does as it happens,
//! rather than reacting afterwards. The core proposes the event ([`Proposal`]), finds the effects that apply, has the
//! affected player order them when several do (CR 616.1), and carries out what is left.
//!
//! An effect is a pair of pure functions over the proposal: `applies` reads the board and the proposal and says
//! whether it replaces this event, `apply` edits the proposal. Neither writes to the game; the core does that once
//! every effect has had its say. Each effect applies at most once to a given event (CR 614.5).

use cantrip_state::*;

use crate::card::Asker;
use crate::game::{Game, Query};

/// An object about to change zones (CR 614.1a, 614.1c, 614.12), as the effects of other permanents see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneChange {
    pub object: ObjectId,
    pub from: Zone,
    /// Where it is going. Setting this is how "exile it instead" and "shuffle it into its library instead" work.
    pub to: Zone,
    pub reason: ZoneReason,
    /// For an object entering the battlefield, the player it enters under (CR 110.2a): the affected player, and the
    /// "you" of an effect about what enters under your control. The object's controller otherwise.
    pub controller: PlayerId,
    /// It enters the battlefield tapped (CR 614.1c).
    pub tapped: bool,
    /// The counters an effect gives it as it arrives, whatever the zone (CR 122.6, 614.1c): the void counter a card
    /// exiled in place of a graveyard carries. Not those a permanent enters with of its own (`Entering`).
    pub counters: [i16; CounterKind::COUNT],
    /// It is revealed as it moves (CR 701.20a).
    pub reveal: bool,
    /// The library it arrives in is shuffled afterwards (CR 701.24a).
    pub shuffle: bool,
    /// It is a card, not a token or a copy of a spell (CR 108.2).
    pub is_card: bool,
}

/// Counters that would be put on a permanent (CR 122.6, 614.1a): by an effect, or as it enters with them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterPlacement {
    pub object: ObjectId,
    /// Its controller, or for a permanent entering, the player it enters under: whose choice CR 616.1 is, and the
    /// "you control" an effect reads.
    pub controller: PlayerId,
    /// The counters are those a permanent enters the battlefield with (CR 122.6): it is not on the battlefield yet and
    /// is judged as it would exist there (CR 614.12).
    pub entering: bool,
    /// How many of each kind would be put.
    pub counts: [i16; CounterKind::COUNT],
    /// The player who puts them (CR 122.6a): who controls the spell or ability, or does the action, doing it. None for
    /// the rules' own doing. "If you would put one or more counters on a permanent or player" is keyed to this.
    pub putter: Option<PlayerId>,
}

/// Counters that would be put on a player (CR 122.1, 122.6): poison counters and energy counters, which a player has
/// rather than an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerCounters {
    pub player: PlayerId,
    pub putter: Option<PlayerId>,
    pub poison: u16,
    pub energy: u16,
}

/// An instruction to draw cards, before any of the individual draws (CR 121.2a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawInstruction {
    pub player: PlayerId,
    pub count: u32,
}

/// One card about to be drawn (CR 121.1, 121.6): replaceable even when the library has no cards to draw. `replaced`
/// is set by an effect that replaces the draw ("instead you win the game"): nothing is drawn then (CR 614.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardDraw {
    pub player: PlayerId,
    pub replaced: bool,
}

/// Damage about to be dealt (CR 614.1a, 120.3): by a source to a player or a permanent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageEvent {
    pub source: ObjectId,
    pub target: Target,
    pub amount: u16,
    pub combat: bool,
    /// The damage already marked on the target is healed as this is dealt: Wolverine's "all other damage already dealt
    /// to him is healed". Damage dealt at the same time as this stays.
    pub heal_marked: bool,
}

/// Tokens about to be created (CR 111.1, 614.1a): `count` of one definition under one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenCreation {
    pub controller: PlayerId,
    pub token: CardId,
    pub count: u32,
}

/// A proposed event a replacement effect may reshape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proposal {
    ZoneChange(ZoneChange),
    Counters(CounterPlacement),
    PlayerCounters(PlayerCounters),
    DrawInstruction(DrawInstruction),
    Draw(CardDraw),
    Damage(DamageEvent),
    TokenCreation(TokenCreation),
}

/// The kinds of event a replacement effect watches for, so an effect is asked only about events it could replace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementKind {
    ZoneChange,
    Counters,
    DrawInstruction,
    Draw,
    Damage,
    TokenCreation,
}

impl Proposal {
    pub fn kind(&self) -> ReplacementKind {
        match self {
            Proposal::ZoneChange(_) => ReplacementKind::ZoneChange,
            Proposal::Counters(_) | Proposal::PlayerCounters(_) => ReplacementKind::Counters,
            Proposal::DrawInstruction(_) => ReplacementKind::DrawInstruction,
            Proposal::Draw(_) => ReplacementKind::Draw,
            Proposal::Damage(_) => ReplacementKind::Damage,
            Proposal::TokenCreation(_) => ReplacementKind::TokenCreation,
        }
    }
}

/// Whether an effect replaces this event, judged on the board as it is. The `Asker` names the effect's source and
/// the player "you" means: its controller, or for an effect about its own entry, the player it enters under.
pub type ReplacementApplies = fn(&dyn Query, &Asker, &Proposal) -> bool;

/// What an effect does to the event.
pub type ReplacementApply = fn(&dyn Query, &Asker, &mut Proposal);

/// What an effect does besides reshaping the event: the work of "instead you create a Treasure token" (CR 614.1a), done
/// to the game once the effect has applied. It is handed the event as it was proposed and as the effect left it, so it
/// can act on the difference (Hullbreacher: one Treasure for each draw the effect removed).
pub type ReplacementThen = fn(&mut dyn Game, &Asker, &Proposal, &Proposal);

/// A replacement effect of a static ability (CR 614.1, 604).
#[derive(Clone, Copy)]
pub struct ReplacementDef {
    /// What it does, in a few words: shown beside its source when a player orders several (CR 616.1), and what a
    /// scenario's `Replace(...)` can name it by.
    pub hint: &'static str,
    pub kind: ReplacementKind,
    /// It functions in every zone, on the move of its own card only: "if this card would be put into a graveyard from
    /// anywhere" (CR 113.6b). Otherwise it functions while its source is on the battlefield (CR 604.1).
    pub own_everywhere: bool,
    pub applies: ReplacementApplies,
    pub apply: ReplacementApply,
    /// Game operations the effect performs as it applies (see [`ReplacementThen`]).
    pub then: Option<ReplacementThen>,
}

/// Whether a replacement effect a spell or ability made (`EffectKind::Replacement`) replaces this event, judged on the board
/// as it is. The `Asker` names the effect's source and its controller; the object is the one it is about.
pub type GrantedApplies = fn(&dyn Query, &Asker, ObjectId, &Proposal) -> bool;

/// What a replacement effect a spell or ability made does to the event.
pub type GrantedApply = fn(&dyn Query, &Asker, ObjectId, &mut Proposal);

/// A replacement effect (CR 614) that a resolving spell or ability makes for a duration (CR 611.2), as opposed to a static
/// ability's (`ReplacementDef`): it is not a card's standing ability, but a continuous effect that lasts, and it is
/// about the object it was made for.
#[derive(Clone, Copy)]
pub struct GrantedReplacementDef {
    /// What it does, in a few words: shown beside its source when a player orders several (CR 616.1).
    pub hint: &'static str,
    pub kind: ReplacementKind,
    pub applies: GrantedApplies,
    pub apply: GrantedApply,
}

fn never(_: &dyn Query, _: &Asker, _: &Proposal) -> bool {
    false
}

fn nothing(_: &dyn Query, _: &Asker, _: &mut Proposal) {}

impl ReplacementDef {
    pub const BLANK: ReplacementDef =
        ReplacementDef { hint: "", kind: ReplacementKind::ZoneChange, own_everywhere: false, applies: never, apply: nothing, then: None };
}
