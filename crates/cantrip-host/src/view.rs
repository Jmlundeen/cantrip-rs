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
//! - `PlayerSummaryView::counters`: not filled.

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

/// What a restriction on mana is, as the wire says it (CR 106.6): what the mana may be spent on, what it does to what it
/// pays for, the subtypes the spell must have one of and the least mana value it may have. Every restriction has its
/// own, so `restriction_of` reads it back.
pub fn restriction_view(restriction: ManaRestriction) -> (ManaSpendFilterView, ManaSpendEffectView, Vec<String>, i32) {
    let mut subtypes = Vec::new();
    let mut mana_value = 0;
    let filter = match restriction {
        ManaRestriction::ManaValue(m) => {
            mana_value = i32::from(m.at_least);
            if m.or_x { ManaSpendFilterView::ManaValueOrXSpells } else { ManaSpendFilterView::ManaValueSpells }
        }
        r if r == ManaRestriction::NONCREATURE_SPELLS => ManaSpendFilterView::NoncreatureSpells,
        ManaRestriction::Subtypes(s) => {
            subtypes = s.subtypes.iter().flatten().map(|t| t.name().to_string()).collect();
            if s.creature { ManaSpendFilterView::SubtypeCreatureSpells } else { ManaSpendFilterView::SubtypeSpells }
        }
        ManaRestriction::NotNonartifactSpells => ManaSpendFilterView::NotNonartifactSpells,
        ManaRestriction::GainsHaste => ManaSpendFilterView::None,
        ManaRestriction::LegendaryUncounterable => ManaSpendFilterView::LegendarySpells,
        r if r == ManaRestriction::ARTIFACT_SPELLS => ManaSpendFilterView::ArtifactSpells,
        r if r == ManaRestriction::INSTANT_OR_SORCERY_SPELLS => ManaSpendFilterView::InstantOrSorcerySpells,
        r if r == ManaRestriction::PLANESWALKER_SPELLS => ManaSpendFilterView::PlaneswalkerSpells,
        r if r == ManaRestriction::ABILITIES => ManaSpendFilterView::ActivatedAbilities,
        r if r == ManaRestriction::LAND_ABILITIES => ManaSpendFilterView::LandAbilities,
        r if r == ManaRestriction::ARTIFACT_ABILITIES => ManaSpendFilterView::ArtifactAbilities,
        r if r == ManaRestriction::CREATURE_SPELLS => ManaSpendFilterView::CreatureSpells,
        r if r == ManaRestriction::CREATURE_ABILITIES => ManaSpendFilterView::CreatureAbilities,
        r if r == ManaRestriction::SPELLS_FROM_EXILE => ManaSpendFilterView::SpellsFromExile,
        r if r == ManaRestriction::NOT_SPELLS_FROM_HAND => ManaSpendFilterView::NotSpellsFromHand,
        r => panic!("a mana restriction the wire has no word for: {}", r.name()),
    };
    let effect = match restriction.effects() {
        MANA_GIVES_HASTE => ManaSpendEffectView::GainsHaste,
        MANA_MAKES_UNCOUNTERABLE => ManaSpendEffectView::CantBeCountered,
        _ => ManaSpendEffectView::None,
    };
    (filter, effect, subtypes, mana_value)
}

/// The restriction a filter, an effect and subtypes from the wire name; none for unrestricted mana.
pub fn restriction_of(filter: ManaSpendFilterView, effect: ManaSpendEffectView, subtypes: &[String], mana_value: i32) -> Option<ManaRestriction> {
    if (filter, effect) == (ManaSpendFilterView::None, ManaSpendEffectView::None) {
        return None;
    }
    if matches!(filter, ManaSpendFilterView::ManaValueSpells | ManaSpendFilterView::ManaValueOrXSpells) {
        let at_least = u8::try_from(mana_value).ok()?;
        return Some(ManaRestriction::ManaValue(ManaValueSpells { at_least, or_x: filter == ManaSpendFilterView::ManaValueOrXSpells }));
    }
    if matches!(filter, ManaSpendFilterView::SubtypeSpells | ManaSpendFilterView::SubtypeCreatureSpells) {
        if subtypes.len() > 2 {
            return None;
        }
        let mut named = [None, None];
        for (slot, name) in named.iter_mut().zip(subtypes) {
            *slot = Some(Subtype::named(name)?);
        }
        return Some(ManaRestriction::Subtypes(SubtypeSpells {
            subtypes: named,
            chosen: false,
            creature: filter == ManaSpendFilterView::SubtypeCreatureSpells,
            uncounterable: effect == ManaSpendEffectView::CantBeCountered,
        }));
    }
    ManaRestriction::NAMED.into_iter().map(|(r, _)| r).find(|&r| restriction_view(r) == (filter, effect, Vec::new(), 0))
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

/// "Hexproof from [quality]" as a card's text words it (CR 702.11d, 702.11f-g): "Hexproof from monocolored", "Hexproof from
/// white and from black", "Hexproof from artifacts, creatures, and enchantments", "Hexproof from each color".
pub fn hexproof_from_text(qualities: HexproofFrom) -> String {
    let mut words: Vec<String> = Vec::new();
    if qualities.colors() == ColorSet(0x1f) {
        words.push("each color".to_string());
    } else {
        words.extend(qualities.colors().iter().map(|c| c.name().to_lowercase()));
    }
    if qualities.has(HexproofFrom::MONOCOLORED) {
        words.push("monocolored".to_string());
    }
    if qualities.has(HexproofFrom::MULTICOLORED) {
        words.push("multicolored".to_string());
    }
    words.extend(qualities.types().iter().map(|t| format!("{}s", t.name().to_lowercase())));
    match words.len() {
        0 => String::new(),
        1 => format!("Hexproof from {}", words[0]),
        2 => format!("Hexproof from {} and from {}", words[0], words[1]),
        n => format!("Hexproof from {}, and {}", words[..n - 1].join(", "), words[n - 1]),
    }
}

/// The keywords as a card's text words them, for the ones an effect gave an object or took from it.
const KEYWORD_WORDS: [(Keyword, &str); 34] = [
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
    (Keyword::Decayed, "Decayed"),
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
        produces_mana: def.abilities.iter().any(|a| matches!(a, AbilityDef::Activated(d) if d.is_mana_ability())),
        starting_counter: def.loyalty.map(i32::from),
        other_face: def.back.map(|back| Box::new(printed(back))),
        halves: def.split.map_or_else(Vec::new, |(left, right)| vec![printed(left), printed(right)]),
        inset_spell: inset_spell(def),
        ..Default::default()
    }
}

/// The Adventure or Omen printed in a card's inset frame (CR 715.2, 720.2), as printed, for a player to read beside the
/// card: it is part of the card in every zone, though the card has only its normal characteristics there (CR 715.4, 720.4).
fn inset_spell(def: &CardDef) -> Option<Box<ObjectView>> {
    def.inset_faces().next().map(|(_, _, face)| Box::new(printed(face)))
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
    // "You may look at the top card of your library any time" (CR 401.2): its player sees it while a permanent says so.
    if game.may_look_at_library_top(viewer)
        && let Some(&top) = s.zone(Zone::library(viewer)).first()
    {
        visible.insert(top);
    }
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
            has_citys_blessing: player.citys_blessing,
            has_enduring_story: player.enduring_story,
            energy: i32::from(player.energy),
            dungeons_completed: i32::from(player.dungeons_completed),
            ..Default::default()
        };
        for mana in Mana::ALL {
            summary.mana[mana as usize] = i32::from(player.pool.get(mana));
        }
        for (restriction, pool) in s.restricted_of(p) {
            for mana in Mana::ALL {
                let n = pool.get(mana);
                if n > 0 {
                    let (filter, effect, spend_subtypes, spend_mana_value) = restriction_view(restriction);
                    summary.restricted_mana.push(RestrictedManaView {
                        r#type: Some(mana_view(mana)),
                        amount: i32::from(n),
                        filter,
                        effect,
                        spend_subtypes,
                        spend_mana_value,
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
        // "Target opponent skips their next X turns" (CR 500.11, 614.10): one effect for each turn still to skip, about the
        // player and on no object.
        let skips = s.effects.iter().filter(|e| e.kind == EffectKind::SkipNextTurn(p)).count();
        if skips > 0 {
            let (who, whose) = if p == self.viewer { ("You", "your") } else { ("They", "their") };
            let turns = if skips == 1 { "turn".to_string() } else { format!("{skips} turns") };
            summary.pending_effects.push(format!("{who} skip {whose} next {turns}."));
        }
        // "Whenever [event] this turn" / "until your next turn, whenever [event]" (CR 603.7b): a delayed triggered ability that
        // waits for its event, on no object, so the page says it beside its controller's life total, in the ability's own words
        // and named for the object it came from (the words may say "that creature"). The other seat is told whose it is. So is
        // one that triggers at each such step for the rest of the game (paradigm, CR 702.192a): the card it came from may be
        // anywhere, or nowhere it can be seen.
        for d in s.delayed.iter().filter(|d| {
            d.controller == p && matches!(d.when, DelayedWhen::ThisTurn | DelayedWhen::UntilNextTurnOf(_) | DelayedWhen::EachStep { .. })
        }) {
            let text = game.cards.ability(d.ability).text();
            let from = game.cards.get(game.chars(d.source).card).name;
            summary.pending_effects.push(if p == self.viewer {
                format!("{from}: {text}")
            } else {
                format!("They have {from}'s delayed ability: {text}")
            });
        }
        // "You may activate loyalty abilities of Jace planeswalkers you control ... any time you could cast an instant"
        // (CR 606.3, 611.2c): a permission about the player, on no object.
        for e in s.effects.iter().filter(|e| e.controller == p) {
            if let EffectKind::InstantLoyalty(kind) = e.kind {
                let they = if p == self.viewer { "you" } else { "they" };
                let walkers = kind.map_or("planeswalkers".to_string(), |t| format!("{} planeswalkers", t.name()));
                let until = if e.duration == Duration::EndOfTurn { " this turn" } else { "" };
                summary.pending_effects.push(format!(
                    "{}{} may activate loyalty abilities of {walkers} {they} control any time {they} could cast an instant{until}.",
                    they[..1].to_uppercase(),
                    &they[1..]
                ));
            }
        }
        // "Your opponents can't cast spells from anywhere other than their hands" until a turn (CR 101.2, 611.2c): a restriction
        // on the player, on no object, so the player it stops is told where they may still cast from.
        for e in s.effects.iter() {
            if let EffectKind::CantCastSpellsFrom(barred, zones) = e.kind
                && barred == p
            {
                summary.pending_effects.push(self.cant_cast_text(p, zones, e.duration));
            }
        }
        // "Players can't gain life" (CR 119.7): from a permanent's static ability while it is on the battlefield, or a resolved
        // effect about the player (Screaming Nemesis's "for the rest of the game"), as it applies now.
        if game.cant_gain_life(p) {
            summary.pending_effects.push(format!("{} can't gain life.", if p == self.viewer { "You" } else { "They" }));
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

    /// "You can't cast spells from anywhere other than your hand until Bot1's next turn", as `viewer` reads a restriction on
    /// `p` (CR 101.2).
    fn cant_cast_text(&self, p: PlayerId, zones: ZoneKinds, duration: Duration) -> String {
        let (who, whose) = if p == self.viewer { ("You", "your") } else { ("They", "their") };
        let open: Vec<ZoneKind> = ZoneKinds::CAST_FROM.into_iter().filter(|&z| !zones.contains(z)).collect();
        let from = match open.as_slice() {
            [] => String::new(),
            [only] => format!(" from anywhere other than {whose} {}", only.name().to_lowercase()),
            _ => {
                let barred: Vec<String> =
                    ZoneKinds::CAST_FROM.into_iter().filter(|&z| zones.contains(z)).map(|z| z.name().to_lowercase()).collect();
                format!(" from {whose} {}", barred.join(" or "))
            }
        };
        let until = match duration {
            Duration::UntilNextTurnOf(q) if q == p && q != self.viewer => format!(" until {whose} next turn"),
            _ => self.until_text(duration),
        };
        format!("{who} can't cast spells{from}{until}.")
    }

    /// How long an effect lasts, as `viewer` reads it: " this turn", " until your next turn", " until Bot1's next turn", or
    /// nothing for one that lasts as long as its object.
    fn until_text(&self, duration: Duration) -> String {
        match duration {
            Duration::EndOfTurn => " this turn".to_string(),
            Duration::UntilNextTurnOf(q) if q == self.viewer => " until your next turn".to_string(),
            Duration::UntilNextTurnOf(q) => format!(" until {}'s next turn", self.names.get(q.index()).cloned().unwrap_or_default()),
            _ => String::new(),
        }
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
        // "Hexproof from [quality]" (CR 702.11d): the qualities it has and did not start with are granted, those it started
        // with and has not are lost.
        if c.hexproof_from != def.hexproof_from {
            let granted = HexproofFrom(c.hexproof_from.0 & !def.hexproof_from.0);
            let lost = HexproofFrom(def.hexproof_from.0 & !c.hexproof_from.0);
            if !granted.is_empty() {
                view.granted_abilities.push(AbilityTextView { text: hexproof_from_text(granted), spent: false });
            }
            if !lost.is_empty() {
                view.lost_abilities.push(hexproof_from_text(lost));
            }
        }
        let printed = |ability: AbilityRef| {
            (ability.card == c.card && usize::from(ability.index) < def.abilities.len())
                // A Room's abilities are its doors' (CR 709.5): the layers give the unlocked ones.
                || cards.halves[c.card.0 as usize].is_some_and(|(left, right)| ability.card == left || ability.card == right)
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
        // A condition the permission stands under is said, so one not met reads as the conditional it is.
        if let Some(PlayCondition::AttackedWithThisTurn(subtype)) = grant.when {
            let who = if mine { "you" } else { "they" };
            text.push_str(&format!(", if {who} attacked with a {}", subtype.name()));
        }
        if grant.sorcery_speed {
            text.push_str(", as a sorcery");
        }
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
            summoning_sick: on_battlefield && summoning_sick(game.cards, s, &game.log, id),
            produces_mana: c.abilities().any(|a| matches!(game.cards.ability(a), AbilityDef::Activated(d) if d.is_mana_ability())),
            monstrous: obj.marks.monstrous,
            solved: on_battlefield && obj.marks.solved,
            suspected: on_battlefield && effect(EffectKind::Suspected),
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
            // Cast as its Adventure or Omen, it is that face alone (CR 715.3b, 720.3b), whose definition has no inset.
            inset_spell: inset_spell(def),
            ..Default::default()
        };

        // A name an effect set (CR 612.8) or a copy effect states (CR 707.9b) is the name it has.
        if c.name_set || obj.copy.is_some_and(|copy| copy.name.is_some()) {
            view.name = c.name.to_string();
        }
        if def.is_room() && on_battlefield && !c.name_set {
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
        // A Case's two keyword lines (CR 719.3): once it is solved its "To solve" ability can do nothing more (CR 719.3a), and
        // its "Solved" ability may affect the game only while it is (CR 719.3c), so the line that can't is greyed.
        if on_battlefield && c.subtypes.has(Subtype::Case) {
            for (i, line) in view.oracle_text.split('\n').enumerate() {
                let idle = if line.starts_with("To solve") {
                    obj.marks.solved
                } else {
                    line.starts_with("Solved") && !obj.marks.solved
                };
                if idle {
                    view.spent_lines.push(i as i32);
                }
            }
        }
        for permission in s.permissions.iter().filter(|p| p.card == id) {
            view.play_permissions.push(PlayPermissionView {
                seat: seat(permission.player),
                text: self.permission_text(permission),
            });
        }
        if (on_battlefield || zone.is(ZoneKind::Stack)) && !obj.face_down {
            self.ability_changes(&mut view, &c, obj.copy);
        }
        // "Can't attack" and "can't block" from a resolved effect (CR 508.1c, 509.1b), with how long they last: no ability
        // of the creature says so, so its face does.
        if on_battlefield {
            for e in s.effects.iter().filter(|e| e.affected == id) {
                let what = match e.kind {
                    EffectKind::CantAttack => "Can't attack",
                    EffectKind::CantBlock => "Can't block",
                    _ => continue,
                };
                view.granted_abilities.push(AbilityTextView { text: format!("{what}{}.", self.until_text(e.duration)), spent: false });
            }
            // A creature with defender that may attack as though it didn't have it right now (CR 702.3b, 609.4): by a resolved
            // effect, with how long it lasts (Assault Formation), or while a static ability's condition holds (Demon Wall with a
            // counter on it). It still has defender, so its face says both.
            if c.has(Keyword::Defender) && c.is(CardType::Creature) {
                const TEXT: &str = "Can attack as though it didn't have defender";
                if let Some(e) = s.effects.iter().find(|e| e.affected == id && e.kind == EffectKind::CanAttackDespiteDefender) {
                    view.granted_abilities.push(AbilityTextView { text: format!("{TEXT}{}.", self.until_text(e.duration)), spent: false });
                } else if game.may_attack_despite_defender(id) {
                    view.granted_abilities.push(AbilityTextView { text: format!("{TEXT}."), spent: false });
                }
            }
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
                view.inset_spell = None;
                view.starting_counter = None;
                // Nothing the layers or the printed card say about it crosses either (CR 406.3): not that it is every creature
                // type, nor what it was granted or has lost.
                view.every_creature_type = false;
                view.type_line_subtypes.clear();
                view.granted_abilities.clear();
                view.lost_abilities.clear();
                view.cant_be_countered = false;
                view.halves.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A power and toughness printed as stars are not a number on the printed card: the goyfs' sizes are their abilities'
    /// (CR 208.2a), and what the stars are worth once that is lost is the ability's (`StaticDef::lost_pt`), so the card as read
    /// in a deck builder or a booster shows none.
    #[test]
    fn a_printed_card_with_stars_for_power_and_toughness_shows_neither() {
        let cards = cantrip_engine::cards();
        for name in ["Tarmogoyf", "Nethergoyf", "Barrowgoyf", "Pyrogoyf"] {
            let view = printed(cards.get(cards.find(name).expect("the card")));
            assert_eq!((view.power, view.toughness), (None, None), "{name}");
        }
    }

    /// "Hexproof from [quality]" granted to a card is worded as printed cards word it (CR 702.11d, 702.11f-g).
    #[test]
    fn hexproof_from_is_worded_as_cards_print_it() {
        let black = HexproofFrom::color(Color::Black);
        let white = HexproofFrom::color(Color::White);
        assert_eq!(hexproof_from_text(HexproofFrom::MONOCOLORED), "Hexproof from monocolored");
        assert_eq!(hexproof_from_text(white.and(black)), "Hexproof from white and from black");
        assert_eq!(hexproof_from_text(HexproofFrom::EACH_COLOR), "Hexproof from each color");
        let types = HexproofFrom::card_type(CardType::Artifact)
            .and(HexproofFrom::card_type(CardType::Creature))
            .and(HexproofFrom::card_type(CardType::Enchantment));
        assert_eq!(hexproof_from_text(types), "Hexproof from artifacts, creatures, and enchantments");
    }
}
