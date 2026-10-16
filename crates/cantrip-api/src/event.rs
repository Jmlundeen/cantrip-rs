//! What happened, as the rules record it: the events triggers listen for and the game log is made of.

use cantrip_state::*;

use crate::history::Cast;

/// One thing that happened. Each names what it is about by id; a zone change names both the old object and the new
/// one it became (CR 400.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A spell became cast (CR 601.2i), with how many spells its caster has cast this turn, this one included.
    SpellCast { spell: ObjectId, player: PlayerId, nth: u8 },
    /// An object or player became the target of a spell or ability (CR 115.10).
    BecameTarget { target: Target, by: ObjectId, controller: PlayerId },
    /// An object moved between zones. A token's creation comes from no zone (`Zone::GONE`).
    ZoneChange { old: ObjectId, new: ObjectId, from: Zone, to: Zone, reason: ZoneReason },
    /// Damage was dealt.
    Damage { source: ObjectId, target: Target, amount: u16, combat: bool },
    LifeChange { player: PlayerId, delta: i32 },
    Drew { player: PlayerId, card: ObjectId },
    Discarded { player: PlayerId, card: ObjectId },
    /// Cards were revealed to every player (CR 701.20a).
    Revealed { player: PlayerId, card: ObjectId },
    Countered { spell: ObjectId },
    AbilityActivated { source: ObjectId, ability: AbilityRef, player: PlayerId },
    /// A creature was declared as an attacker (CR 508.1).
    Attacks { attacker: ObjectId, defender: Target },
    /// A Class became a level (CR 716.2a).
    ClassLevel { object: ObjectId, level: u8 },
    /// A card was cycled (CR 702.29c).
    Cycled { card: ObjectId, player: PlayerId },
    /// A player looked at a card (a look at one card of a hand, at a card exiled face down): kept for the history, and
    /// seen by that player alone.
    Looked { player: PlayerId, card: ObjectId },
    /// A library was searched (CR 701.23).
    /// `player` searched the library of `library` (CR 701.23, 701.23f): their own, or another player's.
    Searched { player: PlayerId, library: PlayerId },
    Tapped { permanent: ObjectId },
    /// A permanent was tapped for mana (CR 605.1b): a mana ability with {T} in its cost was activated. Recorded for the history;
    /// the triggered mana abilities it triggers (`TriggeredDef::mana_ability`) resolve at once, where the tap happens.
    TappedForMana { permanent: ObjectId, player: PlayerId },
    /// A player became the monarch (CR 725.1, 725.4): it was not theirs before.
    BecameMonarch { player: PlayerId },
    /// Creatures were declared as attackers (CR 508.1): "whenever you attack" (CR 508.1m). Not for creatures put onto
    /// the battlefield attacking (CR 508.4).
    AttackersDeclared { player: PlayerId, count: u8 },
    /// A creature was declared as a blocker of an attacker (CR 509.1, 509.1g): one event for each blocker and the
    /// attacker it blocks, so a creature that blocks "this blocks" and an attacker blocked by two sees both.
    Blocks { blocker: ObjectId, attacker: ObjectId },
    /// An attacking creature was not blocked (CR 509.1h): decided as the declare blockers step's blocks are made.
    Unblocked { attacker: ObjectId, defender: Target },
    /// A permanent became monstrous (CR 701.37a).
    Monstrous { permanent: ObjectId },
    StepBegan { step: Step },
    /// Counters were put on a permanent. Only lore counters are announced (CR 714.2b: a Saga's chapters). `by` is the
    /// player who put them (CR 122.6a): the controller of the spell or ability whose effect did it, the player whose
    /// action it was, or the controller of a permanent that enters with them; none for the rules themselves.
    CountersPut { object: ObjectId, kind: CounterKind, amount: i16, by: Option<PlayerId> },
    /// A permanent transformed (CR 701.28): its other face is up now.
    Transformed { object: ObjectId },
    /// `player` gave a gift (CR 702.174c) to `to`: an instant or sorcery spell they control whose gift cost was paid
    /// resolved, or the gift triggered ability of a permanent they control resolved.
    GaveGift { player: PlayerId, to: PlayerId, gift: Gift },
    /// An Aura or Equipment stopped being attached to `from` (CR 701.3d): moved, unattached, or either one left.
    Unattached { attachment: ObjectId, from: ObjectId },
    /// A spell's characteristics as it became cast (CR 601.2i), recorded just before its `SpellCast`: what "the second
    /// creature spell you cast this turn" reads.
    Cast(Cast),
    /// A door of a Room was unlocked (CR 709.5f, 709.5h): the permanent was given the unlocked designation of that half
    /// (`DOOR_LEFT` or `DOOR_RIGHT`) by `player`, whether as it entered because the half was cast (CR 709.5d), by the
    /// special action (CR 709.5e) or by an instruction to unlock.
    DoorUnlocked { permanent: ObjectId, door: u8, player: PlayerId },
    /// A spell was copied (CR 707.10): `copy` is the new spell on the stack, controlled by `player`, who made it. It is
    /// not cast (CR 707.10a), so it is no `SpellCast`: magecraft's "or copy".
    Copied { copy: ObjectId, original: ObjectId, player: PlayerId },
    /// A player flipped a coin (CR 705.1) and won or lost the flip (CR 705.2). A flip is a two-sided die, but no die roll.
    CoinFlipped { player: PlayerId, won: bool },
    /// A player rolled a die with `sides` sides (CR 706.2): the `natural` result, and the `result` once whatever modifies
    /// the roll has been applied (CR 706.4). Both are what the rolled number is, each for the questions that ask for it.
    DieRolled { player: PlayerId, sides: u8, natural: u8, result: i16 },
    /// One of `count` things was chosen at random (CR 701.9c): the one at `index`. A scenario fixes it like a die of `count`
    /// sides (face `index + 1`).
    ChosenAtRandom { count: u16, index: u16 },
    /// An activated or triggered ability of this object finished resolving (CR 608.2m).
    Resolved { source: ObjectId, ability: AbilityRef },
    /// A triggered ability of this object triggered and was put on the stack (CR 603.2h).
    Triggered { source: ObjectId, ability: AbilityRef },
    /// The modes of a modal spell or ability of this object were chosen (CR 601.2b, 602.2b, 603.3c, 700.2), as a
    /// `ModalDef` mask: what "choose one that hasn't been chosen" asks of the history.
    ModesChosen { source: ObjectId, ability: AbilityRef, modes: u8 },
    /// The player did what an ability says they may do only once each turn (CR 603.2h: "do this only once each turn").
    Done { source: ObjectId, ability: AbilityRef, player: PlayerId },
}

impl Event {
    /// The player an event is about, where it is about one.
    pub fn player(&self) -> Option<PlayerId> {
        match *self {
            Event::SpellCast { player, .. }
            | Event::LifeChange { player, .. }
            | Event::Drew { player, .. }
            | Event::Discarded { player, .. }
            | Event::Revealed { player, .. }
            | Event::Looked { player, .. }
            | Event::AbilityActivated { player, .. }
            | Event::Cycled { player, .. }
            | Event::Searched { player, .. }
            | Event::AttackersDeclared { player, .. }
            | Event::TappedForMana { player, .. }
            | Event::BecameMonarch { player }
            | Event::GaveGift { player, .. }
            | Event::Copied { player, .. }
            | Event::DoorUnlocked { player, .. }
            | Event::CoinFlipped { player, .. }
            | Event::DieRolled { player, .. }
            | Event::Done { player, .. } => Some(player),
            Event::Cast(c) => Some(c.player),
            Event::BecameTarget { controller, .. } => Some(controller),
            Event::Damage { target: Target::Player(p), .. } => Some(p),
            _ => None,
        }
    }

    /// Whether the event is about `object`: its subject, a zone change's old or new object, a damage event's source or
    /// target.
    pub fn is_about(&self, object: ObjectId) -> bool {
        match *self {
            Event::SpellCast { spell, .. } | Event::Countered { spell } => spell == object,
            Event::Cast(c) => c.spell == object,
            Event::Copied { copy, original, .. } => copy == object || original == object,
            Event::BecameTarget { target, by, .. } => by == object || target == Target::Object(object),
            Event::ZoneChange { old, new, .. } => old == object || new == object,
            Event::Damage { source, target, .. } => source == object || target == Target::Object(object),
            Event::Drew { card, .. } | Event::Discarded { card, .. } | Event::Revealed { card, .. } | Event::Looked { card, .. } | Event::Cycled { card, .. } => {
                card == object
            }
            Event::AbilityActivated { source, .. }
            | Event::Resolved { source, .. }
            | Event::Triggered { source, .. }
            | Event::ModesChosen { source, .. }
            | Event::Done { source, .. } => source == object,
            Event::Attacks { attacker, .. } | Event::Unblocked { attacker, .. } => attacker == object,
            Event::Blocks { blocker, attacker } => blocker == object || attacker == object,
            Event::ClassLevel { object: o, .. }
            | Event::CountersPut { object: o, .. }
            | Event::Transformed { object: o } => o == object,
            Event::Tapped { permanent }
            | Event::TappedForMana { permanent, .. }
            | Event::Monstrous { permanent }
            | Event::DoorUnlocked { permanent, .. } => permanent == object,
            Event::Unattached { attachment, from } => attachment == object || from == object,
            Event::LifeChange { .. } | Event::Searched { .. } | Event::AttackersDeclared { .. } | Event::StepBegan { .. } | Event::BecameMonarch { .. } | Event::GaveGift { .. } | Event::ChosenAtRandom { .. } | Event::CoinFlipped { .. } | Event::DieRolled { .. } => false,
        }
    }
}

/// The kinds of event, as a bitset, so a triggered ability names what it listens for and an event is checked only
/// against the abilities that could match it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct EventMask(pub u64);

impl EventMask {
    pub const SPELL_CAST: EventMask = EventMask(1 << 0);
    pub const BECAME_TARGET: EventMask = EventMask(1 << 1);
    pub const ZONE_CHANGE: EventMask = EventMask(1 << 2);
    pub const DAMAGE: EventMask = EventMask(1 << 3);
    pub const STEP_BEGAN: EventMask = EventMask(1 << 4);
    pub const DISCARDED: EventMask = EventMask(1 << 5);
    pub const CYCLED: EventMask = EventMask(1 << 6);
    pub const ATTACKS: EventMask = EventMask(1 << 7);
    pub const CLASS_LEVEL: EventMask = EventMask(1 << 8);
    pub const SEARCHED: EventMask = EventMask(1 << 9);
    pub const DREW: EventMask = EventMask(1 << 10);
    pub const ATTACKERS_DECLARED: EventMask = EventMask(1 << 11);
    pub const UNBLOCKED: EventMask = EventMask(1 << 12);
    pub const MONSTROUS: EventMask = EventMask(1 << 13);
    pub const COUNTERS: EventMask = EventMask(1 << 14);
    pub const TRANSFORMED: EventMask = EventMask(1 << 15);
    pub const UNATTACHED: EventMask = EventMask(1 << 30);
    pub const TAPPED: EventMask = EventMask(1 << 29);
    pub const LIFE_CHANGE: EventMask = EventMask(1 << 16);
    pub const REVEALED: EventMask = EventMask(1 << 17);
    pub const COUNTERED: EventMask = EventMask(1 << 18);
    pub const ABILITY_ACTIVATED: EventMask = EventMask(1 << 19);
    pub const CAST: EventMask = EventMask(1 << 20);
    pub const RESOLVED: EventMask = EventMask(1 << 21);
    pub const TRIGGERED: EventMask = EventMask(1 << 22);
    pub const DONE: EventMask = EventMask(1 << 23);
    pub const BLOCKS: EventMask = EventMask(1 << 24);
    pub const COPIED: EventMask = EventMask(1 << 25);
    pub const BECAME_MONARCH: EventMask = EventMask(1 << 26);
    pub const COIN_FLIPPED: EventMask = EventMask(1 << 27);
    pub const DIE_ROLLED: EventMask = EventMask(1 << 28);
    pub const CHOSEN_AT_RANDOM: EventMask = EventMask(1 << 31);
    pub const LOOKED: EventMask = EventMask(1 << 32);
    pub const MODES_CHOSEN: EventMask = EventMask(1 << 33);
    pub const DOOR_UNLOCKED: EventMask = EventMask(1 << 34);
    pub const GAVE_GIFT: EventMask = EventMask(1 << 35);

    /// An event's kind as the history files it: every event has one. `of` is what triggers listen for, which leaves
    /// out the records no ability triggers on.
    pub fn of_history(event: &Event) -> EventMask {
        match event {
            Event::LifeChange { .. } => EventMask::LIFE_CHANGE,
            Event::Revealed { .. } => EventMask::REVEALED,
            Event::Looked { .. } => EventMask::LOOKED,
            Event::Countered { .. } => EventMask::COUNTERED,
            Event::AbilityActivated { .. } => EventMask::ABILITY_ACTIVATED,
            Event::CountersPut { .. } => EventMask::COUNTERS,
            Event::Cast(_) => EventMask::CAST,
            Event::Resolved { .. } => EventMask::RESOLVED,
            Event::Triggered { .. } => EventMask::TRIGGERED,
            Event::ModesChosen { .. } => EventMask::MODES_CHOSEN,
            Event::Done { .. } => EventMask::DONE,
            other => EventMask::of(other),
        }
    }

    pub fn of(event: &Event) -> EventMask {
        match event {
            Event::SpellCast { .. } => EventMask::SPELL_CAST,
            Event::BecameTarget { .. } => EventMask::BECAME_TARGET,
            Event::ZoneChange { .. } => EventMask::ZONE_CHANGE,
            Event::Damage { .. } => EventMask::DAMAGE,
            Event::StepBegan { .. } => EventMask::STEP_BEGAN,
            Event::Discarded { .. } => EventMask::DISCARDED,
            Event::Cycled { .. } => EventMask::CYCLED,
            Event::Attacks { .. } => EventMask::ATTACKS,
            Event::ClassLevel { .. } => EventMask::CLASS_LEVEL,
            Event::Searched { .. } => EventMask::SEARCHED,
            Event::Drew { .. } => EventMask::DREW,
            Event::AttackersDeclared { .. } => EventMask::ATTACKERS_DECLARED,
            Event::Unblocked { .. } => EventMask::UNBLOCKED,
            Event::Blocks { .. } => EventMask::BLOCKS,
            Event::Monstrous { .. } => EventMask::MONSTROUS,
            Event::CountersPut { .. } => EventMask::COUNTERS,
            Event::Transformed { .. } => EventMask::TRANSFORMED,
            Event::Unattached { .. } => EventMask::UNATTACHED,
            Event::Tapped { .. } => EventMask::TAPPED,
            Event::BecameMonarch { .. } => EventMask::BECAME_MONARCH,
            Event::CoinFlipped { .. } => EventMask::COIN_FLIPPED,
            Event::DieRolled { .. } => EventMask::DIE_ROLLED,
            Event::ChosenAtRandom { .. } => EventMask::CHOSEN_AT_RANDOM,
            Event::LifeChange { .. } => EventMask::LIFE_CHANGE,
            Event::Countered { .. } => EventMask::COUNTERED,
            Event::Copied { .. } => EventMask::COPIED,
            Event::DoorUnlocked { .. } => EventMask::DOOR_UNLOCKED,
            Event::GaveGift { .. } => EventMask::GAVE_GIFT,
            _ => EventMask(0),
        }
    }

    pub fn intersects(self, other: EventMask) -> bool {
        self.0 & other.0 != 0
    }
}
