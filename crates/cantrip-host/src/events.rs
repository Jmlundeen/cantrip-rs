//! The engine's log as the entries the client shows.
//!
//! Two stages. `Tracker::collect` reads what happened, once, over the whole game and without regard to who is
//! watching (`Proto`): every native `Event` through the one function `proto_of`, plus what only a comparison of the
//! position before and after can say (an ability put on the stack or resolved, a player lost). Then `Proto::tell`
//! says what one seat may be told of it, in that seat's handles; the same proto has the same sequence number for every
//! seat, so a later retraction can name it.
//!
//! # Mapping
//!
//! | native                                            | `GameEventView`                                     |
//! |---------------------------------------------------|-----------------------------------------------------|
//! | `StepBegan` (and the untap step, read off a wrap)  | `StepBeganEvent`                                    |
//! | `ZoneChange`                                      | `ObjectMovedEvent`, or `TokenCreatedEvent` from nowhere |
//! | `LifeChange`                                      | `LifeChangedEvent`                                  |
//! | `Damage` to an object                             | `DamageMarkedEvent`                                 |
//! | `Tapped`                                          | `TapChangedEvent`                                   |
//! | `SpellCast`                                       | `SpellCastEvent` (targets, X, modes, kicks)         |
//! | `Revealed`                                        | `CardsRevealedEvent`                                |
//! | `CountersPut` (lore only, as the engine reports)  | `CountersPutEvent`                                  |
//! | `Transformed`                                     | `TransformedEvent`                                  |
//! | a new activated/triggered item on the stack       | `AbilityActivatedEvent`                             |
//! | an ability item gone from the stack               | `AbilityResolvedEvent`                              |
//! | a player's `lost`, the outcome                    | `PlayerLostEvent`, `GameEndedEvent`                 |
//! | monarch, extra turns                              | `MonarchBecameEvent`, `ExtraTurnAddedEvent`         |
//! | `DieRolled` (the result, modifiers applied)        | `DieRolledEvent`                                    |
//! | `CoinFlipped`                                     | `CoinFlippedEvent`                                  |
//! | the answers to the opening's questions            | `StartingPlayerChosenEvent`, `MulliganTakenEvent`, `HandKeptEvent` |
//!
//! Not told, because the engine has no event or state for it: untaps, shuffles (`LibraryShuffledEvent`), scry and
//! surveil (`LibraryArrangedEvent`), cycling and discard-as-cost as such (they arrive as moves), extra phases,
//! day and night, a random player chosen (`ChosenAtRandom` is a bare index; the effect that chose says what), fizzling (`SpellFizzledEvent`), retractions (a cast given up is never
//! published), counters other than lore, damage to players as such (life covers it), mana abilities, combat
//! declarations as log lines (combat is in the view).

use std::collections::{HashMap, HashSet};

use cantrip_engine::*;

use crate::handles::HandleSpace;
use crate::protocol::*;
use crate::view::*;

/// What happened, for nobody in particular.
#[derive(Clone, Debug)]
pub enum Proto {
    Step { turn: i32, step: Step, active: PlayerId },
    Moved { old: ObjectId, new: ObjectId, from: Zone, to: Zone, reason: ZoneReason },
    Created { object: ObjectId, controller: PlayerId, name: String },
    Life { player: PlayerId, delta: i32, total: i32 },
    Damage { object: ObjectId, amount: i32 },
    Tap { object: ObjectId, tapped: bool },
    Cast { spell: ObjectId, controller: PlayerId, targets: Vec<Target>, x: Option<i32>, modes: Vec<String>, kicked: i32, gift: Option<PlayerId> },
    AbilityPut { serial: u32, source: ObjectId, controller: PlayerId, text: String, targets: Vec<Target>, triggered: bool, modes: Vec<String> },
    AbilityResolved { serial: u32, source: ObjectId, controller: PlayerId, targets: Vec<Target> },
    Revealed { player: PlayerId, cards: Vec<ObjectId> },
    Counters { object: ObjectId, name: String, count: i32 },
    Transformed { object: ObjectId, from: String, to: String },
    Lost { player: PlayerId, reason: LossReasonView },
    Ended { outcome: GameOutcomeView, winning_team: i32 },
    Monarch { player: PlayerId },
    ExtraTurn { player: PlayerId, count: i32 },
    StartingPlayer { chooser: PlayerId, player: PlayerId },
    Mulligan { player: PlayerId, count: i32 },
    Kept { player: PlayerId, cards: i32 },
    Die { player: PlayerId, sides: i32, result: i32 },
    Coin { player: PlayerId, won: bool },
}

/// The ability a stack item was put on the stack as, kept until it leaves.
#[derive(Clone, Debug)]
struct Seen {
    source: ObjectId,
    controller: PlayerId,
    targets: Vec<Target>,
}

/// What has been told so far, to say what is new.
#[derive(Default)]
pub struct Tracker {
    turn: i32,
    last_step: Option<Step>,
    active: PlayerId,
    life: Vec<i32>,
    stack: HashMap<u32, Seen>,
    lost: Vec<bool>,
    ended: bool,
    monarch: Option<PlayerId>,
    extra_turns: usize,
}

fn spell_facts(game: &Game, spell: ObjectId) -> (Vec<Target>, Option<i32>, Vec<String>, i32, Option<PlayerId>) {
    let item = game.state.stack.iter().rev().find(|i| i.kind == StackKind::Spell && i.object == spell);
    let Some(item) = item else { return (Vec::new(), None, Vec::new(), 0, None) };
    let def = game.cards.get(game.chars(spell).card);
    let mut modes = Vec::new();
    if let Some(modal) = def.modal {
        for (i, mode) in modal.modes.iter().enumerate() {
            if modal.times(item.modes, i) > 0 {
                modes.push(mode.text.to_string());
            }
        }
    }
    let x = (def.cost.x > 0).then_some(i32::from(item.x));
    (item.targets().to_vec(), x, modes, i32::from(item.kicked), item.gift)
}

impl Tracker {
    pub fn new(game: &Game) -> Tracker {
        Tracker {
            turn: game.state.turn.number as i32,
            last_step: None,
            active: game.state.turn.active,
            life: game.state.players.iter().map(|p| p.life).collect(),
            lost: vec![false; game.state.players.len()],
            ..Default::default()
        }
    }

    /// The one place a native event becomes something to tell. Everything else a seat hears is a comparison of
    /// positions (`collect`).
    fn proto_of(&mut self, game: &Game, event: &Event, out: &mut Vec<Proto>) {
        match *event {
            Event::StepBegan { step } => {
                let wrapped = self.last_step.is_none_or(|last| step <= last);
                if wrapped {
                    self.turn += 1;
                    let state = &game.state;
                    self.active = if self.turn == state.turn.number as i32 { state.turn.active } else { state.next_player(self.active) };
                    out.push(Proto::Step { turn: self.turn, step: Step::Untap, active: self.active });
                }
                self.last_step = Some(step);
                out.push(Proto::Step { turn: self.turn, step, active: self.active });
            }
            Event::ZoneChange { old, new, from, to, reason } => {
                if to.is(ZoneKind::Gone) {
                    return;
                }
                if from.is(ZoneKind::Gone) {
                    let name = game.cards.get(game.chars(new).card).name.to_string();
                    out.push(Proto::Created { object: new, controller: game.state.obj(new).controller, name });
                } else {
                    out.push(Proto::Moved { old, new, from, to, reason });
                }
            }
            Event::LifeChange { player, delta } => {
                let total = self.life.get_mut(player.index()).map(|l| {
                    *l += delta;
                    *l
                });
                out.push(Proto::Life { player, delta, total: total.unwrap_or(game.state.player(player).life) });
            }
            Event::Damage { target: Target::Object(object), amount, .. } => {
                out.push(Proto::Damage { object, amount: i32::from(amount) });
            }
            Event::Tapped { permanent } => out.push(Proto::Tap { object: permanent, tapped: true }),
            Event::SpellCast { spell, player, .. } => {
                let (targets, x, modes, kicked, gift) = spell_facts(game, spell);
                out.push(Proto::Cast { spell, controller: player, targets, x, modes, kicked, gift });
            }
            Event::Revealed { player, card } => out.push(Proto::Revealed { player, cards: vec![card] }),
            Event::CountersPut { object, kind, amount, .. } => {
                out.push(Proto::Counters { object, name: kind.name().to_string(), count: i32::from(amount) });
            }
            Event::Transformed { object } => {
                let def = game.cards.get(game.state.obj(object).card);
                let (front, back) = (def.name.to_string(), def.back.map_or(String::new(), |b| b.name.to_string()));
                let up = game.state.obj(object).transformed;
                let (from, to) = if up { (front, back) } else { (back, front) };
                out.push(Proto::Transformed { object, from, to });
            }
            Event::DieRolled { player, sides, result, .. } => {
                out.push(Proto::Die { player, sides: i32::from(sides), result: i32::from(result) });
            }
            Event::CoinFlipped { player, won } => out.push(Proto::Coin { player, won }),
            _ => {}
        }
    }

    /// Everything the game did since the last collection: the log entries in `events`, then what the position says
    /// that they do not.
    pub fn collect(&mut self, game: &Game, events: &[Event]) -> Vec<Proto> {
        let mut out = Vec::new();
        for event in events {
            self.proto_of(game, event, &mut out);
        }
        let state = &game.state;
        self.turn = state.turn.number as i32;
        self.active = state.turn.active;

        // Abilities on the stack: new ones were put there, missing ones resolved (or were countered).
        let mut now: HashSet<u32> = HashSet::new();
        for item in state.stack.iter().filter(|i| i.kind != StackKind::Spell) {
            now.insert(item.serial);
            if !self.stack.contains_key(&item.serial) {
                let text = game.cards.ability(item.ability).text().to_string();
                self.stack.insert(item.serial, Seen { source: item.object, controller: item.controller, targets: item.targets().to_vec() });
                out.push(Proto::AbilityPut {
                    serial: item.serial,
                    source: item.object,
                    controller: item.controller,
                    text,
                    targets: item.targets().to_vec(),
                    triggered: item.kind == StackKind::Triggered,
                    modes: Vec::new(),
                });
            }
        }
        let mut gone: Vec<u32> = self.stack.keys().copied().filter(|s| !now.contains(s)).collect();
        gone.sort();
        for serial in gone {
            if let Some(seen) = self.stack.remove(&serial) {
                out.push(Proto::AbilityResolved { serial, source: seen.source, controller: seen.controller, targets: seen.targets });
            }
        }

        for p in state.seats() {
            let player = state.player(p);
            if player.lost && !self.lost[p.index()] {
                self.lost[p.index()] = true;
                let reason = if player.life <= 0 {
                    LossReasonView::Life
                } else if player.poison >= 10 {
                    LossReasonView::Poison
                } else if player.drew_from_empty {
                    LossReasonView::EmptyLibrary
                } else {
                    LossReasonView::Effect
                };
                out.push(Proto::Lost { player: p, reason });
            }
            if player.monarch && self.monarch != Some(p) {
                self.monarch = Some(p);
                out.push(Proto::Monarch { player: p });
            }
        }
        if state.extra_turns.len() > self.extra_turns {
            for p in &state.extra_turns[self.extra_turns..] {
                out.push(Proto::ExtraTurn { player: *p, count: 1 });
            }
        }
        self.extra_turns = state.extra_turns.len();

        if state.is_over() && !self.ended {
            self.ended = true;
            out.push(match state.outcome {
                Outcome::Won(p) => Proto::Ended { outcome: GameOutcomeView::TeamWon, winning_team: seat(p) },
                _ => Proto::Ended { outcome: GameOutcomeView::Draw, winning_team: 0 },
            });
        }
        out
    }

    /// A seat conceded: the host marks the loss, and says so.
    pub fn conceded(&mut self, player: PlayerId) -> Vec<Proto> {
        if let Some(l) = self.lost.get_mut(player.index()) {
            *l = true;
        }
        vec![Proto::Lost { player, reason: LossReasonView::Conceded }]
    }
}

fn reason_view(reason: ZoneReason, from: Zone, to: Zone) -> ZoneChangeReasonView {
    use ZoneChangeReasonView as V;
    match reason {
        ZoneReason::Draw => V::Draw,
        ZoneReason::Discard => V::Discard,
        ZoneReason::Mill => V::Milled,
        ZoneReason::Sacrifice => V::Sacrificed,
        ZoneReason::Destroy | ZoneReason::DestroyNoRegen => V::Destroyed,
        ZoneReason::PlayLand => V::Play,
        ZoneReason::Cast => V::Cast,
        ZoneReason::Resolve => V::Resolve,
        ZoneReason::Counter => V::Countered,
        ZoneReason::StateBased => V::StateBasedAction,
        ZoneReason::Exile => V::Exiled,
        ZoneReason::Created => V::Created,
        ZoneReason::Search => V::Searched,
        ZoneReason::Other => match (from.kind(), to.kind()) {
            (ZoneKind::Stack, _) => V::Resolve,
            (_, ZoneKind::Hand) => V::PutIntoHand,
            (_, ZoneKind::Library) => V::PutOnLibrary,
            (_, ZoneKind::Exile) => V::Exiled,
            (ZoneKind::Battlefield, _) => V::StateBasedAction,
            _ => V::Returned,
        },
    }
}

/// What one seat is working with while it is told a batch.
pub struct Teller<'a> {
    pub game: &'a Game,
    pub viewer: PlayerId,
    pub handles: &'a mut HandleSpace,
    /// Objects shown to this seat right now (a reveal, a prompt).
    pub extra: &'a HashSet<ObjectId>,
    /// Every object an event named: their handles outlive this projection by one cycle.
    pub mentioned: HashSet<ObjectId>,
}

impl Teller<'_> {
    fn visible(&self, id: ObjectId) -> bool {
        let s = &self.game.state;
        if is_ability_key(id) {
            return true;
        }
        if self.extra.contains(&id) {
            return true;
        }
        id.index() < s.objects.len() && live(s, id) && looks_into(self.viewer, s.obj(id).zone)
    }

    /// Minted for what the viewer can observe now, looked up for what they cannot.
    fn handle_for(&mut self, id: ObjectId) -> ViewHandle {
        self.mentioned.insert(id);
        if self.visible(id) { self.handles.handle(id) } else { self.handles.existing(id) }
    }

    fn target(&mut self, t: Target) -> Option<TargetMessage> {
        match t {
            Target::Player(p) => Some(TargetMessage::player(seat(p))),
            Target::Object(id) => {
                let h = self.handle_for(id);
                (!h.is_none()).then(|| TargetMessage::object(h))
            }
            Target::Ability(serial) => Some(TargetMessage::object(self.handles.handle(ability_key(serial)))),
            Target::None => None,
        }
    }

    pub fn tell(&mut self, proto: &Proto) -> Option<GameEventView> {
        match proto {
            Proto::Step { turn, step, active } => {
                Some(GameEventView::StepBeganEvent(StepBeganEvent { turn: *turn, step: step_view(*step), active_player: seat(*active) }))
            }
            Proto::Moved { old, new, from, to, reason } => {
                let owner = self.game.state.obj(*new).owner;
                let to_seen = looks_into(self.viewer, Zone { player: if matches!(to.kind(), ZoneKind::Hand) { owner } else { to.player }, ..*to });
                let existing = self.handles.existing(*old);
                self.mentioned.insert(*old);
                self.mentioned.insert(*new);
                let object = if to_seen || self.extra.contains(new) {
                    if existing.is_none() {
                        self.handles.handle(*new)
                    } else {
                        self.handles.rebind(*old, *new);
                        existing
                    }
                } else {
                    existing
                };
                Some(GameEventView::ObjectMovedEvent(ObjectMovedEvent {
                    object,
                    from: zone_kind_view(from.kind()),
                    from_owner: zone_owner(*from),
                    to: zone_kind_view(to.kind()),
                    to_owner: zone_owner(*to),
                    reason: reason_view(*reason, *from, *to),
                }))
            }
            Proto::Created { object, controller, name } => {
                let h = self.handle_for(*object);
                Some(GameEventView::TokenCreatedEvent(TokenCreatedEvent { object: h, controller: seat(*controller), name: name.clone() }))
            }
            Proto::Life { player, delta, total } => {
                Some(GameEventView::LifeChangedEvent(LifeChangedEvent { player: seat(*player), delta: *delta, new_total: *total }))
            }
            Proto::Damage { object, amount } => {
                let h = self.handle_for(*object);
                (!h.is_none()).then_some(GameEventView::DamageMarkedEvent(DamageMarkedEvent { object: h, amount: *amount }))
            }
            Proto::Tap { object, tapped } => {
                let h = self.handle_for(*object);
                (!h.is_none()).then_some(GameEventView::TapChangedEvent(TapChangedEvent { object: h, tapped: *tapped }))
            }
            Proto::Cast { spell, controller, targets, x, modes, kicked, gift } => {
                let spell = self.handle_for(*spell);
                let targets = targets.iter().filter_map(|t| self.target(*t)).collect();
                Some(GameEventView::SpellCastEvent(SpellCastEvent {
                    spell,
                    controller: seat(*controller),
                    targets,
                    x: *x,
                    modes: modes.clone(),
                    kicked: *kicked,
                    buyback: false,
                    gift_promised_to: gift.map(seat),
                }))
            }
            Proto::AbilityPut { serial, source, controller, text, targets, triggered, modes } => {
                let ability = self.handles.handle(ability_key(*serial));
                let source = self.handle_for(*source);
                let targets = targets.iter().filter_map(|t| self.target(*t)).collect();
                Some(GameEventView::AbilityActivatedEvent(AbilityActivatedEvent {
                    ability,
                    source,
                    controller: seat(*controller),
                    text: text.clone(),
                    targets,
                    triggered: *triggered,
                    modes: modes.clone(),
                }))
            }
            Proto::AbilityResolved { serial, source, controller, targets } => {
                let ability = self.handles.existing(ability_key(*serial));
                let source = self.handle_for(*source);
                let targets = targets.iter().filter_map(|t| self.target(*t)).collect();
                Some(GameEventView::AbilityResolvedEvent(AbilityResolvedEvent {
                    ability,
                    source,
                    controller: seat(*controller),
                    first: 0,
                    targets,
                    power_toughness: Vec::new(),
                }))
            }
            Proto::Revealed { player, cards } => {
                let cards = cards.iter().map(|c| self.handle_for(*c)).collect();
                Some(GameEventView::CardsRevealedEvent(CardsRevealedEvent { player: seat(*player), cards }))
            }
            Proto::Counters { object, name, count } => {
                let h = self.handle_for(*object);
                (!h.is_none()).then(|| GameEventView::CountersPutEvent(CountersPutEvent { object: h, name: name.clone(), count: *count }))
            }
            Proto::Transformed { object, from, to } => {
                let h = self.handle_for(*object);
                (!h.is_none()).then(|| GameEventView::TransformedEvent(TransformedEvent { object: h, from: from.clone(), to: to.clone() }))
            }
            Proto::Lost { player, reason } => Some(GameEventView::PlayerLostEvent(PlayerLostEvent { player: seat(*player), reason: *reason })),
            Proto::Ended { outcome, winning_team } => {
                Some(GameEventView::GameEndedEvent(GameEndedEvent { outcome: *outcome, winning_team: *winning_team }))
            }
            Proto::Monarch { player } => Some(GameEventView::MonarchBecameEvent(MonarchBecameEvent { player: seat(*player) })),
            Proto::ExtraTurn { player, count } => {
                Some(GameEventView::ExtraTurnAddedEvent(ExtraTurnAddedEvent { player: seat(*player), count: *count }))
            }
            Proto::StartingPlayer { chooser, player } => {
                Some(GameEventView::StartingPlayerChosenEvent(StartingPlayerChosenEvent { chooser: seat(*chooser), player: seat(*player) }))
            }
            Proto::Mulligan { player, count } => {
                Some(GameEventView::MulliganTakenEvent(MulliganTakenEvent { player: seat(*player), count: *count }))
            }
            Proto::Die { player, sides, result } => {
                Some(GameEventView::DieRolledEvent(DieRolledEvent { player: seat(*player), sides: *sides, result: *result }))
            }
            Proto::Coin { player, won } => Some(GameEventView::CoinFlippedEvent(CoinFlippedEvent { player: seat(*player), won: *won })),
            Proto::Kept { player, cards } => Some(GameEventView::HandKeptEvent(HandKeptEvent { player: seat(*player), cards: *cards })),
        }
    }
}
