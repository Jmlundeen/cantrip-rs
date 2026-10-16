//! A seat's `PlayerView` from the native engine's state: what that seat may see and nothing else.
//!
//! Visibility follows the C# projector (`Cantrip.Projection.PlayerViewProjector`): the battlefield, the stack, the
//! graveyards, exile and the command zone to everyone, a hand to its owner, a library to nobody (CR 401.2). Cards a
//! prompt or a reveal is showing right now cross to the seat they are shown to (`extra`), as the C# `alsoReveal` does:
//! they are objects of the view with handles, though the zone they sit in stays closed.
//!
//! Characteristics are computed (`Game::chars`), never printed: power, toughness, types, colors and subtypes come out
//! of the layers, so an anthem shows.
//!
//! # Left at the default, because the engine has no notion of it
//!
//! - `ObjectView::fingerprint`, `spent_lines`, `known_by` and `ZoneView::known`: the engine keeps
//!   no record of who has been shown which hidden card, so only a reveal of this step crosses (as an object).
//! - `SpellTargetView::text` is empty and `still_legal` always true.
//! - Face-down permanents (morph, manifest) are not in the engine yet.
//! - `PlayerSummaryView::counters` and `has_citys_blessing`: the engine has neither.

use std::collections::HashSet;

use cantrip_engine::*;

use crate::handles::HandleSpace;
use crate::protocol::*;

/// The key an ability on the stack has in a handle space: abilities are no objects of the engine, but a client points
/// at one (to target it, to order it), so each is named by the serial it was put on the stack with.
pub fn ability_key(serial: u32) -> ObjectId {
    ObjectId(0x8000_0000 | serial)
}

pub fn is_ability_key(id: ObjectId) -> bool {
    id.0 & 0x8000_0000 != 0
}

/// Seat number of a player: one-based.
pub fn seat(player: PlayerId) -> i32 {
    i32::from(player.0) + 1
}

/// The player at a seat number, or none for zero.
pub fn player_at(seat: i32) -> Option<PlayerId> {
    (seat > 0).then(|| PlayerId((seat - 1) as u8))
}

/// Whether `viewer` may look into a zone.
pub fn looks_into(viewer: PlayerId, zone: Zone) -> bool {
    match zone.kind() {
        ZoneKind::Hand => zone.player == viewer,
        ZoneKind::Library | ZoneKind::Gone => false,
        _ => true,
    }
}

pub fn zone_kind_view(kind: ZoneKind) -> ZoneKindView {
    match kind {
        ZoneKind::Library => ZoneKindView::Library,
        ZoneKind::Hand => ZoneKindView::Hand,
        ZoneKind::Graveyard => ZoneKindView::Graveyard,
        ZoneKind::Exile => ZoneKindView::Exile,
        ZoneKind::Battlefield => ZoneKindView::Battlefield,
        ZoneKind::Stack => ZoneKindView::Stack,
        ZoneKind::Command => ZoneKindView::Command,
        // Where a token that left, or an object that never was, would be: the closest thing the wire has.
        ZoneKind::Gone => ZoneKindView::Exile,
    }
}

/// Seat owning a zone, or zero for a shared one.
pub fn zone_owner(zone: Zone) -> i32 {
    match zone.kind() {
        ZoneKind::Library | ZoneKind::Hand | ZoneKind::Graveyard => seat(zone.player),
        _ => 0,
    }
}

pub fn step_view(step: Step) -> TurnStepView {
    match step {
        Step::Untap => TurnStepView::Untap,
        Step::Upkeep => TurnStepView::Upkeep,
        Step::Draw => TurnStepView::Draw,
        Step::PrecombatMain => TurnStepView::PrecombatMain,
        Step::BeginCombat => TurnStepView::BeginCombat,
        Step::DeclareAttackers => TurnStepView::DeclareAttackers,
        Step::DeclareBlockers => TurnStepView::DeclareBlockers,
        Step::CombatDamage => TurnStepView::CombatDamage,
        Step::EndOfCombat => TurnStepView::EndOfCombat,
        Step::PostcombatMain => TurnStepView::PostcombatMain,
        Step::End => TurnStepView::End,
        Step::Cleanup => TurnStepView::Cleanup,
    }
}

pub fn color_view(color: Color) -> ColorView {
    match color {
        Color::White => ColorView::White,
        Color::Blue => ColorView::Blue,
        Color::Black => ColorView::Black,
        Color::Red => ColorView::Red,
        Color::Green => ColorView::Green,
    }
}

pub fn color_of(view: ColorView) -> Color {
    match view {
        ColorView::White => Color::White,
        ColorView::Blue => Color::Blue,
        ColorView::Black => Color::Black,
        ColorView::Red => Color::Red,
        ColorView::Green => Color::Green,
    }
}

pub fn mana_view(mana: Mana) -> ManaTypeView {
    match mana {
        Mana::W => ManaTypeView::White,
        Mana::U => ManaTypeView::Blue,
        Mana::B => ManaTypeView::Black,
        Mana::R => ManaTypeView::Red,
        Mana::G => ManaTypeView::Green,
        Mana::C => ManaTypeView::Colorless,
    }
}

pub fn mana_of(view: ManaTypeView) -> Mana {
    match view {
        ManaTypeView::White => Mana::W,
        ManaTypeView::Blue => Mana::U,
        ManaTypeView::Black => Mana::B,
        ManaTypeView::Red => Mana::R,
        ManaTypeView::Green => Mana::G,
        ManaTypeView::Colorless => Mana::C,
    }
}

pub fn colors_view(set: ColorSet) -> Vec<ColorView> {
    set.iter().map(color_view).collect()
}

/// The order a type line prints card types in.
const TYPE_ORDER: [(CardType, CardTypeView); 9] = [
    (CardType::Kindred, CardTypeView::Kindred),
    (CardType::Artifact, CardTypeView::Artifact),
    (CardType::Enchantment, CardTypeView::Enchantment),
    (CardType::Land, CardTypeView::Land),
    (CardType::Planeswalker, CardTypeView::Planeswalker),
    (CardType::Battle, CardTypeView::Battle),
    (CardType::Creature, CardTypeView::Creature),
    (CardType::Instant, CardTypeView::Instant),
    (CardType::Sorcery, CardTypeView::Sorcery),
];

pub fn types_view(types: TypeSet) -> Vec<CardTypeView> {
    TYPE_ORDER.iter().filter(|(t, _)| types.has(*t)).map(|(_, v)| *v).collect()
}

pub fn supertypes_view(s: Supertypes) -> Vec<SupertypeView> {
    let mut out = Vec::new();
    if s.legendary {
        out.push(SupertypeView::Legendary);
    }
    if s.basic {
        out.push(SupertypeView::Basic);
    }
    if s.snow {
        out.push(SupertypeView::Snow);
    }
    out
}

pub fn subtypes_view(set: SubtypeSet) -> Vec<String> {
    set.iter().map(|t| t.name().to_string()).collect()
}

/// The keywords as a card's text words them, for the ones an effect gave an object or took from it.
const KEYWORD_WORDS: [(Keyword, &str); 33] = [
    (Keyword::Flying, "Flying"),
    (Keyword::Haste, "Haste"),
    (Keyword::Vigilance, "Vigilance"),
    (Keyword::Trample, "Trample"),
    (Keyword::FirstStrike, "First strike"),
    (Keyword::DoubleStrike, "Double strike"),
    (Keyword::Deathtouch, "Deathtouch"),
    (Keyword::Lifelink, "Lifelink"),
    (Keyword::Reach, "Reach"),
    (Keyword::Defender, "Defender"),
    (Keyword::Flash, "Flash"),
    (Keyword::Hexproof, "Hexproof"),
    (Keyword::Indestructible, "Indestructible"),
    (Keyword::Menace, "Menace"),
    (Keyword::Infect, "Infect"),
    (Keyword::Shadow, "Shadow"),
    (Keyword::Shroud, "Shroud"),
    (Keyword::CantBeCountered, "This spell can't be countered."),
    (Keyword::CantBlock, "This creature can't block."),
    (Keyword::CanBlockOnlyCreaturesWithFlying, "This creature can block only creatures with flying."),
    (Keyword::DoesNotUntap, "This permanent doesn't untap during your untap step."),
    (Keyword::CrewsAsThoughPowerTwoGreater, "This creature crews Vehicles as though its power were 2 greater."),
    (Keyword::CantBeBlocked, "This creature can't be blocked."),
    (Keyword::CantSpendManaToCast, "You can't spend mana to cast this spell."),
    (Keyword::CastFromYourGraveyard, "You may cast this card from your graveyard."),
    (Keyword::Retrace, "Retrace"),
    (Keyword::Daybound, "Daybound"),
    (Keyword::Nightbound, "Nightbound"),
    (Keyword::SplitSecond, "Split second"),
    (Keyword::CantBeAttacked, "This permanent can't be attacked."),
    (Keyword::CantBeBlockedByPowerTwoOrLess, "This creature can't be blocked by creatures with power 2 or less."),
    (Keyword::CantBeBlockedExceptByThreeOrMore, "This creature can't be blocked except by three or more creatures."),
    (Keyword::Changeling, "Changeling"),
];

/// A mana cost in oracle notation; empty for no mana cost at all (CR 202.1b).
pub fn cost_text(def: &CardDef) -> String {
    if def.has_cost { def.cost.to_string() } else { String::new() }
}

/// One printed card as a card to be read: its name, cost, type line, text and printed power and toughness, and its
/// colors from its cost and color indicator (CR 202.2). No handle, no state.
pub fn printed(def: &CardDef) -> ObjectView {
    ObjectView {
        handle: ViewHandle::NONE,
        known: true,
        card_id: def.name.to_string(),
        name: def.name.to_string(),
        mana_cost: cost_text(def),
        mana_value: if def.has_cost { def.cost.mana_value() as i32 } else { 0 },
        oracle_text: def.text.to_string(),
        supertypes: supertypes_view(def.supertypes),
        card_types: types_view(def.types),
        subtypes: subtypes_view(def.subtypes),
        colors: colors_view(def.colors()),
        power: def.power.map(i32::from),
        toughness: def.toughness.map(i32::from),
        produces_mana: def.abilities.iter().any(|a| matches!(a, AbilityDef::Activated(d) if d.mana.is_some())),
        starting_counter: def.loyalty.map(i32::from),
        other_face: def.back.map(|back| Box::new(printed(back))),
        halves: def.split.map_or_else(Vec::new, |(left, right)| vec![printed(left), printed(right)]),
        ..Default::default()
    }
}

/// One entry of the stack, bottom first.
#[derive(Clone, Copy, Debug)]
pub enum StackEntry {
    /// A spell (its object), and the item announced for it once it is paid for.
    Spell { id: ObjectId, item: Option<StackItem> },
    /// An activated or triggered ability, named by its serial.
    Ability { item: StackItem },
}

impl StackEntry {
    /// What a handle space keys it by.
    pub fn key(&self) -> ObjectId {
        match self {
            StackEntry::Spell { id, .. } => *id,
            StackEntry::Ability { item } => ability_key(item.serial),
        }
    }
}

/// The stack as the rules have it, bottom first: its items, then a spell still being announced (on the stack by its
/// zone, with no item yet).
pub fn stack_entries(state: &GameState) -> Vec<StackEntry> {
    let mut out: Vec<StackEntry> = Vec::new();
    for item in &state.stack {
        match item.kind {
            StackKind::Spell => out.push(StackEntry::Spell { id: item.object, item: Some(*item) }),
            _ => out.push(StackEntry::Ability { item: *item }),
        }
    }
    for &id in state.zone(Zone::STACK) {
        if !out.iter().any(|e| matches!(e, StackEntry::Spell { id: s, .. } if *s == id)) {
            out.push(StackEntry::Spell { id, item: None });
        }
    }
    out
}

/// Every object this viewer can observe: the contents of the zones they may look into, every spell and ability on the
/// stack, and what is being shown to them right now (`extra`).
pub fn observable(game: &Game, viewer: PlayerId, extra: &HashSet<ObjectId>) -> HashSet<ObjectId> {
    let s = &game.state;
    let mut visible = HashSet::new();
    for p in s.seats() {
        if p == viewer {
            visible.extend(s.zone(Zone::hand(p)).iter().copied());
        }
        visible.extend(s.zone(Zone::graveyard(p)).iter().copied());
        visible.extend(s.zone(Zone::exile(p)).iter().copied());
    }
    visible.extend(s.battlefield().iter().copied());
    visible.extend(s.zone(Zone::COMMAND).iter().copied());
    visible.extend(stack_entries(s).iter().map(StackEntry::key));
    for &id in extra {
        if !is_ability_key(id) && (id.index() < s.objects.len()) && live(s, id) {
            visible.insert(id);
        }
    }
    visible
}

fn target_view(game: &Game, handles: &mut HandleSpace, target: Target) -> Option<TargetMessage> {
    match target {
        Target::Player(p) => Some(TargetMessage::player(seat(p))),
        Target::Object(id) => Some(TargetMessage::object(handles.handle(id))),
        Target::Ability(serial) => {
            let _ = game;
            Some(TargetMessage::object(handles.handle(ability_key(serial))))
        }
        Target::None => None,
    }
}

pub struct Projector<'a> {
    pub game: &'a Game,
    pub viewer: PlayerId,
    pub names: &'a [String],
    /// Seats that conceded: they read as such rather than as lost.
    pub conceded: &'a [bool],
}

/// `viewer`'s view of the game. `extra` are objects shown to the viewer right now, and `mentioned` objects whose
/// handles must survive this projection because the events going out beside it name them.
pub fn player_view(
    game: &Game,
    viewer: PlayerId,
    handles: &mut HandleSpace,
    names: &[String],
    conceded: &[bool],
    extra: &HashSet<ObjectId>,
    mentioned: &HashSet<ObjectId>,
) -> PlayerView {
    let projector = Projector { game, viewer, names, conceded };
    projector.build(handles, extra, mentioned)
}

impl<'a> Projector<'a> {
    fn build(&self, handles: &mut HandleSpace, extra: &HashSet<ObjectId>, mentioned: &HashSet<ObjectId>) -> PlayerView {
        let game = self.game;
        let s = &game.state;
        let visible = observable(game, self.viewer, extra);
        // Before anything is minted: whatever this viewer can no longer see loses its handle, so if it ever comes back
        // it comes back unrecognisable.
        handles.retain(|o| visible.contains(&o) || mentioned.contains(&o));

        let pending = game.pending();
        let mut view = PlayerView {
            viewer: seat(self.viewer),
            turn: s.turn.number as i32,
            step: Some(step_view(s.turn.step())),
            active_player: seat(s.turn.active),
            priority_player: match pending {
                Some(r) if matches!(r.kind, RequestKind::Priority { .. }) => seat(r.player),
                _ => 0,
            },
            day_night: DayNightView::Neither,
            ..Default::default()
        };
        match s.outcome {
            Outcome::InProgress => {}
            Outcome::Draw => view.outcome = GameOutcomeView::Draw,
            Outcome::Won(p) => {
                view.outcome = GameOutcomeView::TeamWon;
                view.winning_team = seat(p);
            }
        }

        for p in s.seats() {
            view.players.push(self.summarise(p));
        }

        // In id order so the wire bytes depend on the position and not on map enumeration order.
        let mut ordered: Vec<ObjectId> = visible.iter().copied().collect();
        ordered.sort();
        let entries = stack_entries(s);
        for id in ordered {
            let handle = handles.handle(id);
            if is_ability_key(id) {
                if let Some(StackEntry::Ability { item }) = entries.iter().find(|e| e.key() == id) {
                    let described = self.describe_ability(item, handle, handles);
                    view.objects.push(described);
                }
                continue;
            }
            let item = entries.iter().find_map(|e| match e {
                StackEntry::Spell { id: s, item } if *s == id => *item,
                _ => None,
            });
            let described = self.describe(id, handle, item, handles);
            view.objects.push(described);
        }
        view.objects.sort_by_key(|o| o.handle);

        for p in s.seats() {
            view.zones.push(self.zone_view(ZoneKind::Library, p, handles, &visible));
            view.zones.push(self.zone_view(ZoneKind::Hand, p, handles, &visible));
            view.zones.push(self.zone_view(ZoneKind::Graveyard, p, handles, &visible));
        }
        for kind in [ZoneKind::Battlefield, ZoneKind::Stack, ZoneKind::Exile, ZoneKind::Command] {
            view.zones.push(self.zone_view(kind, PlayerId(0), handles, &visible));
        }
        view.combat = self.combat(handles);
        view
    }

    fn summarise(&self, p: PlayerId) -> PlayerSummaryView {
        let game = self.game;
        let s = &game.state;
        let player = s.player(p);
        let mut summary = PlayerSummaryView {
            seat: seat(p),
            name: self.names.get(p.index()).cloned().unwrap_or_default(),
            team: seat(p),
            life: player.life,
            status: if self.conceded.get(p.index()).copied().unwrap_or(false) {
                PlayerStatusView::Conceded
            } else if player.lost {
                PlayerStatusView::Lost
            } else {
                PlayerStatusView::Active
            },
            poison_counters: i32::from(player.poison),
            hand_count: s.zone(Zone::hand(p)).len() as i32,
            library_count: s.zone(Zone::library(p)).len() as i32,
            lands_played_this_turn: i32::from(player.lands_played),
            is_monarch: player.monarch,
            has_initiative: player.initiative,
            energy: i32::from(player.energy),
            dungeons_completed: i32::from(player.dungeons_completed),
            ..Default::default()
        };
        for mana in Mana::ALL {
            summary.mana[mana as usize] = i32::from(player.pool.get(mana));
        }
        for (i, pool) in player.restricted.iter().enumerate() {
            for mana in Mana::ALL {
                let n = pool.get(mana);
                if n > 0 {
                    summary.restricted_mana.push(RestrictedManaView {
                        r#type: Some(mana_view(mana)),
                        amount: i32::from(n),
                        filter: if i == ManaRestriction::NotNonartifactSpells as usize {
                            ManaSpendFilterView::NotNonartifactSpells
                        } else if i == ManaRestriction::ArtifactSpells as usize {
                            ManaSpendFilterView::ArtifactSpells
                        } else if i == ManaRestriction::InstantOrSorcerySpells as usize {
                            ManaSpendFilterView::InstantOrSorcerySpells
                        } else {
                            ManaSpendFilterView::None
                        },
                        effect: ManaSpendEffectView::None,
                    });
                }
            }
        }
        // "The next spell you cast this turn can't be countered" (CR 611.2c): an effect about the player, one per activation,
        // that sits on no object.
        let waiting = s.effects.iter().filter(|e| e.controller == p && e.kind == EffectKind::NextSpellCantBeCountered).count();
        for _ in 0..waiting {
            let whose = if p == self.viewer { "you" } else { "they" };
            summary.pending_effects.push(format!("The next spell {whose} cast this turn can't be countered."));
        }
        if player.room != 0
            && let Some(dungeon) = game.cards.undercity
        {
            let label = label_text(&Label::Card(dungeon, player.room - 1));
            summary.dungeon = "Undercity".into();
            summary.dungeon_room = label.split(" \u{2014} ").next().unwrap_or(label).to_string();
        }
        summary
    }

    fn zone_view(&self, kind: ZoneKind, owner: PlayerId, handles: &mut HandleSpace, visible: &HashSet<ObjectId>) -> ZoneView {
        let s = &self.game.state;
        let mut contents: Vec<ObjectId> = match kind {
            ZoneKind::Library => s.zone(Zone::library(owner)).to_vec(),
            ZoneKind::Hand => s.zone(Zone::hand(owner)).to_vec(),
            ZoneKind::Graveyard => s.zone(Zone::graveyard(owner)).to_vec(),
            ZoneKind::Battlefield => s.battlefield().to_vec(),
            ZoneKind::Stack => stack_entries(s).iter().map(StackEntry::key).collect(),
            ZoneKind::Exile => s.seats().flat_map(|p| s.zone(Zone::exile(p)).to_vec()).collect(),
            ZoneKind::Command => s.zone(Zone::COMMAND).to_vec(),
            ZoneKind::Gone => Vec::new(),
        };
        let count = contents.len() as i32;
        let zone = Zone::new(kind, owner);
        let contents_visible = looks_into(self.viewer, zone);
        let mut view = ZoneView {
            kind: Some(zone_kind_view(kind)),
            owner: zone_owner(zone),
            count,
            contents_visible,
            ..Default::default()
        };
        if contents_visible {
            contents.retain(|id| visible.contains(id));
            view.contents = contents.into_iter().map(|id| handles.handle(id)).collect();
        }
        view
    }

    fn combat(&self, handles: &mut HandleSpace) -> CombatView {
        let s = &self.game.state;
        let mut view = CombatView::default();
        for a in &s.attackers {
            let (target_kind, target_player, target_permanent) = match a.target {
                Target::Player(p) => (AttackTargetKindView::Player, seat(p), ViewHandle::NONE),
                Target::Object(id) => {
                    let kind = if self.game.chars(id).is(CardType::Battle) {
                        AttackTargetKindView::Battle
                    } else {
                        AttackTargetKindView::Planeswalker
                    };
                    (kind, 0, handles.handle(id))
                }
                _ => (AttackTargetKindView::Player, 0, ViewHandle::NONE),
            };
            view.attackers.push(AttackerView {
                attacker: handles.handle(a.id),
                target_kind,
                target_player,
                target_permanent,
                was_blocked: a.blocked,
            });
        }
        for a in &s.attackers {
            let blockers: Vec<ViewHandle> = s.blocks.iter().filter(|b| b.attacker == a.id).map(|b| handles.handle(b.blocker)).collect();
            if !blockers.is_empty() {
                view.blocks.push(BlockView { attacker: handles.handle(a.id), blockers });
            }
        }
        view
    }

    /// An ability on the stack, as a client draws it: the source's name and the ability's text.
    fn describe_ability(&self, item: &StackItem, handle: ViewHandle, handles: &mut HandleSpace) -> ObjectView {
        let game = self.game;
        let source = game.state.obj(item.object);
        let def = game.cards.get(game.chars(item.object).card);
        let text = game.cards.ability(item.ability).text();
        let mut view = ObjectView {
            handle,
            zone: Some(ZoneKindView::Stack),
            kind: ObjectKindView::AbilityOnStack,
            owner: seat(source.owner),
            controller: seat(item.controller),
            known: true,
            card_id: def.name.to_string(),
            name: def.name.to_string(),
            oracle_text: text.to_string(),
            ..Default::default()
        };
        for t in item.targets() {
            if let Some(message) = target_view(game, handles, *t) {
                view.targets.push(SpellTargetView {
                    player: message.player,
                    object: message.object,
                    text: String::new(),
                    still_legal: true,
                });
            }
        }
        view
    }

    /// What the layers gave an object's abilities and took from them (CR 613.1f), against the abilities it started with: the
    /// card's own, or a copy's. A keyword or ability it has that it did not start with is granted, and one it started with
    /// and has not is lost. The abilities a land has from its basic land types are the rules' and are neither. Nothing is
    /// allocated for an object the layers left alone, which is nearly every one.
    fn ability_changes(&self, view: &mut ObjectView, c: &Chars, copy: Option<Copiable>) {
        let cards = self.game.cards;
        let def = cards.get(c.card);
        let base = Keywords(def.keywords.0 | copy.map_or(0, |copy| copy.extra_keywords.0));
        if c.keywords != base {
            for (keyword, words) in KEYWORD_WORDS {
                if c.keywords.has(keyword) && !base.has(keyword) {
                    view.granted_abilities.push(AbilityTextView { text: words.to_string(), spent: false });
                } else if base.has(keyword) && !c.keywords.has(keyword) {
                    view.lost_abilities.push(words.to_string());
                }
            }
        }
        let printed = |ability: AbilityRef| {
            (ability.card == c.card && usize::from(ability.index) < def.abilities.len())
                || copy.is_some_and(|copy| copy.extra_abilities.contains(&Some(ability)))
        };
        for ability in c.abilities.iter().flatten().copied() {
            if ability.card != cards.intrinsic && !printed(ability) {
                view.granted_abilities.push(AbilityTextView { text: cards.ability(ability).text().to_string(), spent: false });
            }
        }
        for index in 0..def.abilities.len().min(c.abilities.len()) {
            let ability = AbilityRef { card: c.card, index: index as u8 };
            if !c.abilities.contains(&Some(ability)) {
                view.lost_abilities.push(def.abilities[index].text().to_string());
            }
        }
    }

    /// A standing permission to play a card, as the viewer reads it: who may, what, and until when.
    fn permission_text(&self, permission: &PlayPermission) -> String {
        let mine = permission.player == self.viewer;
        let (prefix, whose) = if mine {
            (String::new(), "your".to_string())
        } else {
            let name = self.names.get(permission.player.index()).cloned().unwrap_or_default();
            (format!("{name}: "), "their".to_string())
        };
        let grant = permission.grant;
        let window = match grant.expiry {
            Expiry::Never => "while it stays here".to_string(),
            Expiry::EndOfTurn => "this turn".to_string(),
            Expiry::EndOfYourNextTurn => format!("until {whose} next turn ends"),
            Expiry::UntilYourNextEndStep => format!("until {whose} next end step"),
        };
        let mut text = format!("{prefix}{} {window}", if grant.lands { "Playable" } else { "Castable" });
        if grant.later_turns {
            text.push_str(", on a later turn");
        }
        if grant.exile_after {
            text.push_str(", then exiled");
        }
        text
    }

    fn describe(&self, id: ObjectId, handle: ViewHandle, item: Option<StackItem>, handles: &mut HandleSpace) -> ObjectView {
        let game = self.game;
        let s = &game.state;
        let obj = s.obj(id);
        let c = game.chars(id);
        let def = game.cards.get(c.card);
        let on_battlefield = obj.zone.is(ZoneKind::Battlefield);
        let zone = if item.is_some() { Zone::STACK } else { obj.zone };

        let mut counters: Vec<CounterView> = CounterKind::ALL
            .iter()
            .filter(|k| obj.counters(**k) > 0)
            .map(|k| CounterView { name: k.name().to_string(), count: i32::from(obj.counters(*k)) })
            .collect();
        counters.sort_by(|a, b| a.name.cmp(&b.name));

        let mut choices = Vec::new();
        if let Some(color) = obj.chosen_color {
            choices.push(ChoiceView { key: "color".into(), value: color.name().to_string() });
        }
        if let Some(sub) = obj.chosen_subtype {
            choices.push(ChoiceView { key: "creature type".into(), value: sub.name().to_string() });
        }
        if let Some(p) = obj.marks.chosen_player {
            choices.push(ChoiceView { key: "player".into(), value: seat(p).to_string() });
        }
        if let Some(card) = obj.marks.chosen_card {
            choices.push(ChoiceView { key: "name".into(), value: game.cards.get(card).name.to_string() });
        }
        choices.sort_by(|a, b| a.key.cmp(&b.key));

        let effect = |kind: EffectKind| s.effects.iter().any(|e| e.affected == id && e.kind == kind);
        let mut view = ObjectView {
            handle,
            zone: Some(zone_kind_view(zone.kind())),
            zone_owner: zone_owner(zone),
            kind: if obj.card_copy {
                ObjectKindView::CardCopy
            } else if obj.token {
                ObjectKindView::Token
            } else if zone.is(ZoneKind::Command) {
                ObjectKindView::Emblem
            } else {
                ObjectKindView::Card
            },
            owner: seat(obj.owner),
            controller: seat(obj.controller),
            known: true,
            card_id: def.name.to_string(),
            name: def.name.to_string(),
            mana_cost: cost_text(def),
            mana_value: Query::mana_value(game, id) as i32,
            oracle_text: crate::text::changed_text(def.text, def.name, Query::words(game, id)),
            supertypes: supertypes_view(c.supertypes),
            card_types: types_view(c.types),
            subtypes: subtypes_view(c.subtypes),
            colors: colors_view(c.colors),
            power: c.power.map(i32::from),
            toughness: c.toughness.map(i32::from),
            tapped: obj.tapped,
            damage_marked: i32::from(obj.damage),
            counters,
            summoning_sick: on_battlefield && summoning_sick(game.cards, s, id),
            produces_mana: c.abilities().any(|a| matches!(game.cards.ability(a), AbilityDef::Activated(d) if d.mana.is_some())),
            monstrous: obj.marks.monstrous,
            exerted: obj.marks.exerted_by.is_some(),
            saddled: game.is_saddled(id),
            goaded: effect(EffectKind::Goaded),
            harnessed: effect(EffectKind::Harnessed),
            choices,
            class_level: if on_battlefield { i32::from(obj.class_level) } else { 0 },
            // A token whose text was changed as it was made has copiable values of its own and is no copy (CR 612.4).
            is_copy: obj.copy.is_some_and(|c| Copiable { words: Words::NONE, ..c } != Copiable::of(obj.card)) || obj.card_copy,
            starting_counter: if on_battlefield { None } else { def.loyalty.map(i32::from) },
            other_face: def.back.map(|back| Box::new(printed(back))),
            // A split card as it is printed, its halves side by side (CR 709.1); on the stack only the half cast exists (CR 709.3b).
            halves: if zone.is(ZoneKind::Stack) { Vec::new() } else { def.split.map_or_else(Vec::new, |(left, right)| vec![printed(left), printed(right)]) },
            ..Default::default()
        };

        if def.is_room() && on_battlefield {
            // A Room on the battlefield (CR 709.5) is its unlocked doors: their names, costs and text; a locked door is
            // one the client can show as locked.
            let doors = obj.marks.doors;
            view.name = c.name.to_string();
            // With neither door unlocked there is no mana cost at all (CR 709.5), not {0}.
            view.mana_cost = if doors & (DOOR_LEFT | DOOR_RIGHT) == 0 { String::new() } else { def.room_cost(doors).to_string() };
            // The doors' text, read through the words it has now (CR 612).
            let text = def.unlocked_doors(doors).map(|half| half.text).collect::<Vec<_>>().join("
");
            view.oracle_text = crate::text::changed_text(&text, def.name, Query::words(game, id));
            view.unlocked_doors = vec![doors & DOOR_LEFT != 0, doors & DOOR_RIGHT != 0];
        }

        // Every creature type (Changeling): the type line prints the types the card has of its own, and says the rest.
        if (c.is(CardType::Creature) || c.is(CardType::Kindred))
            && Subtype::CREATURE_TYPES.iter().all(|&t| c.subtypes.has(t))
        {
            view.every_creature_type = true;
            view.type_line_subtypes =
                c.subtypes.iter().filter(|t| !t.is_creature_type() || def.subtypes.has(*t)).map(|t| t.name().to_string()).collect();
        }
        view.cant_be_countered = obj.marks.cant_be_countered;
        for permission in s.permissions.iter().filter(|p| p.card == id) {
            view.play_permissions.push(PlayPermissionView {
                seat: seat(permission.player),
                text: self.permission_text(permission),
            });
        }
        if (on_battlefield || zone.is(ZoneKind::Stack)) && !obj.face_down {
            self.ability_changes(&mut view, &c, obj.copy);
        }

        if obj.face_down {
            // Face down (CR 708.2a, 406.3): what it is face down goes to every seat, a nameless 2/2 on the battlefield and no
            // characteristics at all in exile; what it really is only to a seat that may look at it, its controller
            // for a permanent (CR 708.5) and those who saw it go there for a card (CR 406.3).
            let looks = if on_battlefield { obj.controller == self.viewer } else { obj.seen_by & (1 << self.viewer.0) != 0 };
            if !on_battlefield {
                view.name = String::new();
                view.mana_cost = String::new();
                view.mana_value = 0;
                view.oracle_text = String::new();
                view.supertypes.clear();
                view.card_types.clear();
                view.subtypes.clear();
                view.colors.clear();
                view.power = None;
                view.toughness = None;
                view.produces_mana = false;
                view.other_face = None;
                view.starting_counter = None;
            }
            let real = game.cards.get(obj.card).name;
            view.face_down = true;
            view.is_copy = obj.card_copy;
            view.known = looks;
            view.card_id = if looks { real.to_string() } else { String::new() };
            view.face_down_name = if looks { real.to_string() } else { String::new() };
        }
        if on_battlefield && let Some(host) = game.attached_to(id) {
            view.attached_to = handles.handle(host);
        }
        if obj.zone.is(ZoneKind::Exile)
            && let Some(link) = obj.exiled
            && live(s, link.by)
            && s.obj(link.by).zone.is(ZoneKind::Battlefield)
        {
            view.exiled_with = handles.handle(link.by);
        }
        if let Some(item) = item {
            for t in item.targets() {
                if let Some(message) = target_view(game, handles, *t) {
                    view.targets.push(SpellTargetView {
                        player: message.player,
                        object: message.object,
                        text: String::new(),
                        still_legal: true,
                    });
                }
            }
            if let Some(modal) = def.modal {
                for (i, mode) in modal.modes.iter().enumerate() {
                    if modal.times(item.modes, i) > 0 {
                        view.chosen_modes.push(mode.text.to_string());
                    }
                }
            }
        }
        view
    }
}
