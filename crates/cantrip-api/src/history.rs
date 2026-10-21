//! The game's event history (the C# engine's `IEventHistory`, CR 603.10 and every "this turn" or "this game"
//! question): every rules event since the game began, in order, each with its serial, the turn it happened in and what
//! caused it. Append-only: nothing behind the head is ever edited, so a question asked of the history before some
//! serial has one answer however often it is asked.
//!
//! It is the open-ended record one-off mechanics ask about the past with ("the second creature spell you cast this
//! turn", "a permanent you controlled left the battlefield this turn", "cards put into your graveyard from the
//! battlefield this turn"), so no such question needs a field of its own in the state. A record names objects and
//! players and copies nothing out of them: what an object was is its row, which is never deleted, and for one that
//! left the battlefield its last-known snapshot (CR 608.2h).
//!
//! **When an event is in the history.** An event is recorded after the triggers it causes have been found and after
//! any replacement of it has been applied: a trigger matching an event, and a replacement deciding whether to replace
//! one, both see the history as it was just before that event. So a question such as "is this the first card they
//! draw in their draw step?" reads the same from a triggered ability (Orcish Bowmasters) and from a replacement effect
//! (Hullbreacher), and the questions cards share are written once, as functions over `History`, in
//! `cantrip_mechanics::history`.

use cantrip_state::*;

use crate::event::{Event, EventMask};

/// A spell as it was cast (CR 601.2i): its characteristics on the stack, and the zone it came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cast {
    /// The spell's object on the stack.
    pub spell: ObjectId,
    pub player: PlayerId,
    pub types: TypeSet,
    pub colors: ColorSet,
    /// Its mana value on the stack (CR 202.3e: X counted).
    pub value: u8,
    /// The zone it was cast from (CR 601.2a).
    pub from: ZoneKind,
    /// Which of the card's own options it was cast by (`ActionOption::index`): 0 its mana cost, n its nth printed way, or
    /// another face's way number.
    pub way: u8,
    /// What let it be cast from where it was, beyond the rules every player has (`ActionOption::granted_by`, CR 601.3): the
    /// permanent whose permission it was (Lurrus of the Dream-Den, Mystic Forge), the spell or ability that gave one
    /// (Snapcaster Mage's), or the card's own retrace. "Once during each of your turns" and "if you cast a spell this way"
    /// ask it, through `granted_by`. `Grant::NONE` where nothing did.
    pub by: Grant,
    /// What its card's `CardDef::cast_fact` said of the board as it became cast ("if you controlled a Faerie as you cast this
    /// spell"); 0 for a card with none. A copy of a spell is not cast (CR 707.10) and has no `Cast`, so `fact_as_cast` is 0
    /// for it.
    pub fact: u32,
}

impl Cast {
    /// What let it be cast from where it was (`by`), if anything beyond the rules did.
    pub fn granted_by(&self) -> Option<Grant> {
        self.by.some()
    }

    pub fn is(&self, t: CardType) -> bool {
        self.types.has(t)
    }
}

/// What made an event happen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cause {
    /// The rules themselves: a state-based action, a trigger put on the stack, the game's setup.
    Rules,
    /// A turn-based action (CR 703): drawing for the turn, untapping, combat damage.
    Turn,
    /// A player's own action or the costs of it (CR 601, 602, 305).
    Player(PlayerId),
    /// A spell resolving.
    Spell(ObjectId),
    /// An activated or triggered ability resolving.
    Ability { source: ObjectId, ability: AbilityRef },
}

/// One thing that happened, as the history keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// Its place in the history: dense, from 0, never reused.
    pub serial: u32,
    /// The turn it happened in, the step of that turn, and whose turn it was.
    pub turn: u32,
    pub step: Step,
    pub active: PlayerId,
    pub cause: Cause,
    pub event: Event,
}

/// A question about the history: every condition that is set must hold.
#[derive(Clone, Copy, Debug, Default)]
pub struct EventQuery {
    /// The kinds of event; none means any kind.
    pub kinds: EventMask,
    /// The player the event is about (`Event::player`).
    pub player: Option<PlayerId>,
    /// The object the event is about (`Event::object`): a zone change matches its old or its new object.
    pub object: Option<ObjectId>,
    /// For a zone change, why it happened.
    pub reason: Option<ZoneReason>,
    /// For a zone change, the kind of zone it left.
    pub from: Option<ZoneKind>,
    /// For a zone change, the kind of zone it went to.
    pub to: Option<ZoneKind>,
    /// For a life change, whether it was a gain (true) or a loss (false).
    pub gained: Option<bool>,
    /// Only this turn's events.
    pub this_turn: bool,
    /// Only events during this step (of whichever turn the other conditions allow).
    pub step: Option<Step>,
    /// Only events during this player's turns.
    pub in_turn_of: Option<PlayerId>,
    /// Only events after this serial.
    pub after: Option<u32>,
}

impl EventQuery {
    pub fn kind(kinds: EventMask) -> EventQuery {
        EventQuery { kinds, ..EventQuery::default() }
    }

    pub fn this_turn(self) -> EventQuery {
        EventQuery { this_turn: true, ..self }
    }

    pub fn player(self, player: PlayerId) -> EventQuery {
        EventQuery { player: Some(player), ..self }
    }

    pub fn object(self, object: ObjectId) -> EventQuery {
        EventQuery { object: Some(object), ..self }
    }

    pub fn reason(self, reason: ZoneReason) -> EventQuery {
        EventQuery { reason: Some(reason), ..self }
    }

    pub fn from(self, from: ZoneKind) -> EventQuery {
        EventQuery { from: Some(from), ..self }
    }

    pub fn to(self, to: ZoneKind) -> EventQuery {
        EventQuery { to: Some(to), ..self }
    }

    pub fn gained(self, gained: bool) -> EventQuery {
        EventQuery { gained: Some(gained), ..self }
    }

    pub fn step(self, step: Step) -> EventQuery {
        EventQuery { step: Some(step), ..self }
    }

    pub fn in_turn_of(self, player: PlayerId) -> EventQuery {
        EventQuery { in_turn_of: Some(player), ..self }
    }

    pub fn after(self, serial: u32) -> EventQuery {
        EventQuery { after: Some(serial), ..self }
    }

    fn matches(&self, r: &Record) -> bool {
        let e = &r.event;
        (self.kinds.0 == 0 || EventMask::of_history(e).intersects(self.kinds))
            && self.player.is_none_or(|p| e.player() == Some(p))
            && self.object.is_none_or(|o| e.is_about(o))
            && self.after.is_none_or(|s| r.serial > s)
            && self.step.is_none_or(|s| r.step == s)
            && self.in_turn_of.is_none_or(|p| r.active == p)
            && match *e {
                Event::ZoneChange { from, to, reason, .. } => {
                    self.reason.is_none_or(|x| x == reason)
                        && self.from.is_none_or(|x| from.is(x))
                        && self.to.is_none_or(|x| to.is(x))
                }
                _ => self.reason.is_none() && self.from.is_none() && self.to.is_none(),
            }
            && match *e {
                Event::LifeChange { delta, .. } => self.gained.is_none_or(|g| g == (delta > 0)),
                _ => self.gained.is_none(),
            }
    }
}

thread_local! {
    /// How many times the history has been read since `take_reads` was last asked (`History::new`, `note_read`).
    static READS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// The history is being read: a question about the past is being asked, whose answer may differ as the history grows.
/// Counted for the rules that find a loop of mandatory actions (CR 104.4b, `cantrip_rules::loops`): two positions alike in
/// everything but their history are the same position only if nothing asked the history in between. Every `History` is
/// made by `History::new`, which notes it; a rule that reads the records some other way notes it itself.
pub fn note_read() {
    READS.with(|r| r.set(r.get().wrapping_add(1)));
}

/// How many times the history has been read on this thread since this was last asked, and starts the count again.
pub fn take_reads() -> u32 {
    READS.with(|r| r.replace(0))
}

/// The history as card code reads it: every record, and this turn's. Made by `History::new`, which notes the read.
#[derive(Clone, Copy)]
pub struct History<'a> {
    pub records: &'a [Record],
    /// The turn being played.
    pub turn: u32,
}

impl<'a> History<'a> {
    /// The history of a game, `records` its every record and `turn` the turn being played: a read of it (`note_read`).
    pub fn new(records: &'a [Record], turn: u32) -> History<'a> {
        note_read();
        History { records, turn }
    }

    /// Every record since the game began, oldest first.
    pub fn all(&self) -> &'a [Record] {
        self.records
    }

    /// This turn's records, oldest first: the tail of the history, found from the end.
    pub fn this_turn(&self) -> &'a [Record] {
        let start = self.records.iter().rposition(|r| r.turn != self.turn).map_or(0, |i| i + 1);
        &self.records[start..]
    }

    /// The records that answer a question, oldest first.
    pub fn select(&self, q: EventQuery) -> impl DoubleEndedIterator<Item = &'a Record> + 'a {
        let scope = if q.this_turn { self.this_turn() } else { self.records };
        scope.iter().filter(move |r| q.matches(r))
    }

    pub fn count(&self, q: EventQuery) -> usize {
        self.select(q).count()
    }

    pub fn any(&self, q: EventQuery) -> bool {
        self.select(q).next().is_some()
    }

    /// The serial the next record will have: an anchor for "since now".
    pub fn head(&self) -> u32 {
        self.records.len() as u32
    }

    // --- The questions cards ask most ---------------------------------------------------------------------------

    /// The spells cast this turn, in order, with their characteristics as cast.
    pub fn casts(&self) -> impl Iterator<Item = &'a Cast> + 'a {
        self.this_turn().iter().filter_map(|r| match &r.event {
            Event::Cast(c) => Some(c),
            _ => None,
        })
    }

    /// The spells `player` cast this turn.
    pub fn casts_by(&self, player: PlayerId) -> impl Iterator<Item = &'a Cast> + 'a {
        self.casts().filter(move |c| c.player == player)
    }

    /// The spells `player` has cast this game (CR 601.2i).
    pub fn casts_this_game(&self, player: PlayerId) -> usize {
        self.records.iter().filter(|r| matches!(&r.event, Event::Cast(c) if c.player == player)).count()
    }

    /// How the spell on the stack as `spell` was cast, if it was cast this turn.
    pub fn cast_of(&self, spell: ObjectId) -> Option<&'a Cast> {
        self.casts().find(|c| c.spell == spell)
    }

    /// What the board was as the spell on the stack as `spell` was cast, by its card's `CardDef::cast_fact` (CR 601.2i): 0 for
    /// a copy, which was not cast (CR 707.10). A spell is cast and resolves in the same turn (the stack is empty before a turn
    /// can end, and an effect that ends it exiles the stack, CR 724.1b), so this turn's casts are where it is.
    pub fn fact_as_cast(&self, spell: ObjectId) -> u32 {
        self.this_turn()
            .iter()
            .rev()
            .find_map(|r| match &r.event {
                Event::Cast(c) if c.spell == spell => Some(c.fact),
                _ => None,
            })
            .unwrap_or(0)
    }

    /// The mana spent to cast the spell on the stack as `spell` (CR 601.2h), by the source that produced it (CR 106.3): each
    /// source with the mana of it spent, its total cost's whole (additional costs too), once for each part of the payment it
    /// gave to. Nothing for a copy, which was not cast and had no mana spent on it (CR 707.10), or for a spell cast without
    /// spending mana. A spell is cast and paid for in one turn, so this turn's records hold it.
    pub fn mana_spent_to_cast(&self, spell: ObjectId) -> impl Iterator<Item = (ObjectId, ManaTally)> + 'a {
        self.this_turn().iter().filter_map(move |r| match r.event {
            Event::ManaSpent { source, on, cast: true, mana, .. } if on == spell => Some((source, mana)),
            _ => None,
        })
    }

    /// Whether mana `source` produced was spent to cast `spell` (CR 106.3, 601.2h): "whenever you cast a spell using mana
    /// produced by [this]". `source` is the object as it produced the mana: one that has left and come back is another (CR
    /// 400.7).
    pub fn cast_using_mana_from(&self, spell: ObjectId, source: ObjectId) -> bool {
        self.mana_spent_to_cast(spell).any(|(from, _)| from == source)
    }

    /// How many times this ability of this object has resolved this turn (CR 608.2m).
    pub fn resolutions(&self, source: ObjectId, ability: AbilityRef) -> usize {
        self.this_turn()
            .iter()
            .filter(|r| matches!(r.event, Event::Resolved { source: s, ability: a } if s == source && a == ability))
            .count()
    }

    /// How many times this ability of this object has triggered this turn (CR 603.2h).
    pub fn triggerings(&self, source: ObjectId, ability: AbilityRef) -> usize {
        self.this_turn()
            .iter()
            .filter(|r| matches!(r.event, Event::Triggered { source: s, ability: a } if s == source && a == ability))
            .count()
    }

    /// Whether the player has done what this ability says they may do only once each turn (CR 603.2h).
    pub fn done(&self, source: ObjectId, ability: AbilityRef) -> bool {
        self.this_turn().iter().any(|r| matches!(r.event, Event::Done { source: s, ability: a, .. } if s == source && a == ability))
    }

    /// Cards `player` has drawn this turn (CR 121).
    pub fn drawn(&self, player: PlayerId) -> usize {
        self.count(EventQuery::kind(EventMask::DREW).player(player).this_turn())
    }

    /// Life `player` has gained this turn (CR 119.3).
    pub fn life_gained(&self, player: PlayerId) -> u32 {
        self.life(player, true)
    }

    /// Life `player` has lost this turn (CR 119.3).
    pub fn life_lost(&self, player: PlayerId) -> u32 {
        self.life(player, false)
    }

    fn life(&self, player: PlayerId, gained: bool) -> u32 {
        self.select(EventQuery::kind(EventMask::LIFE_CHANGE).player(player).gained(gained).this_turn())
            .map(|r| match r.event {
                Event::LifeChange { delta, .. } => delta.unsigned_abs(),
                _ => 0,
            })
            .sum()
    }

    /// Whether `object` was put into a graveyard by a discard this turn (CR 701.9): "if you discarded it this turn".
    pub fn discarded_this_turn(&self, object: ObjectId) -> bool {
        self.this_turn().iter().any(|r| {
            matches!(r.event, Event::ZoneChange { new, to, reason: ZoneReason::Discard, .. } if new == object && to.is(ZoneKind::Graveyard))
        })
    }

    /// Whether `creature` was declared as an attacker this turn (CR 508.1; not one put onto the battlefield attacking).
    pub fn attacked_this_turn(&self, creature: ObjectId) -> bool {
        self.this_turn().iter().any(|r| matches!(r.event, Event::Attacks { attacker, .. } if attacker == creature))
    }
}
