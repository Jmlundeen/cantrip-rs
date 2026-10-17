//! Characteristics (CR 613), computed when asked: the copiable values (layer 1, CR 707), then the continuous
//! effects that apply to the object in timestamp order or as they depend on each other (CR 613.7, 613.8), then counters
//! (layer 7c).
//!
//! Characteristics are not cached per object: a game has a few dozen objects and a handful of effects. What is shared
//! is what does not depend on the object: a card's own values (`CardTable::printed`) and the list of static-ability
//! sources, found once per position (`GameState::epoch`, which every write and undo moves).
//!
//! Every entry point takes the event history beside the state (`log`): a static ability's condition may ask about the
//! past ("as long as you've lost life this turn", `LayerQuery::history`), and its effect is not locked in (CR 611.3a), so it
//! is read afresh with everything else. Recording an event does not move the epoch, and nothing cached here depends on the
//! history: which permanents have static abilities is a question of the state alone.

use cantrip_api::*;
use cantrip_state::ContinuousEffect;

pub fn chars(cards: &CardTable, state: &GameState, log: &[Record], id: ObjectId) -> Chars {
    chars_before(cards, state, log, id, Layer::End)
}

/// An object's characteristics as if it were the face `face` of its card (CR 715.4, 709.3, 712.8a): a card in hand
/// judged as the Adventure, Omen, half or back face it would be cast as. The face's own printed values replace the card's.
pub fn chars_as_face(cards: &CardTable, state: &GameState, log: &[Record], id: ObjectId, face: CardId) -> Chars {
    let mut chars = chars(cards, state, log, id);
    let printed = initial(cards, face, None);
    chars.card = face;
    chars.name = printed.name;
    chars.types = printed.types;
    chars.supertypes = printed.supertypes;
    chars.subtypes = printed.subtypes;
    chars.colors = printed.colors;
    chars.power = printed.power;
    chars.toughness = printed.toughness;
    chars.keywords = printed.keywords;
    chars.abilities = printed.abilities;
    chars.spell = true;
    chars
}

/// What a static ability's functions read: other objects through the layers before the one being applied.
struct View<'a> {
    cards: &'a CardTable,
    state: &'a GameState,
    /// The event history (`cantrip_api::history`), which a static ability's condition may ask about the past.
    log: &'a [Record],
    stop: Layer,
}

impl LayerQuery for View<'_> {
    fn state(&self) -> &GameState {
        self.state
    }

    fn cards(&self) -> &CardTable {
        self.cards
    }

    fn history(&self) -> History<'_> {
        History { records: self.log, turn: self.state.turn.number }
    }

    fn chars(&self, id: ObjectId) -> Chars {
        chars_before(self.cards, self.state, self.log, id, self.stop)
    }

    fn chars_with_defined_pt(&self, id: ObjectId) -> Chars {
        chars_before(self.cards, self.state, self.log, id, Layer::PtSet)
    }
}

/// More permanents with static abilities than any game has at once; past it, the rest are not looked at.
const MAX_STATIC_SOURCES: usize = 32;

const LAYERS: [Layer; 8] =
    [Layer::Text, Layer::Type, Layer::Color, Layer::Ability, Layer::PtDefine, Layer::PtSet, Layer::PtModify, Layer::PtSwitch];

/// The characteristics with every layer before `stop` applied (CR 613.1): the copiable values, then each layer's
/// static abilities and other continuous effects in turn, in timestamp order unless one depends on another (CR 613.7, 613.8).
pub fn chars_before(cards: &CardTable, state: &GameState, log: &[Record], id: ObjectId, stop: Layer) -> Chars {
    let obj = state.obj(id);
    let spell = obj.zone.is(ZoneKind::Stack);
    // The copiable values (CR 707.2): a copy effect's, or the face of the card that is up (CR 711.2).
    let card = match &obj.copy {
        Some(c) => c.card,
        None => cards.face(obj.card, obj.transformed),
    };
    let def = cards.get(card);
    // A Room on the battlefield (CR 709.5): the name, mana cost and abilities of its unlocked doors alone, which no
    // card-only cache can hold, as they are the object's. In every other zone the card has both halves (CR 709.4).
    let room = def.is_room() && obj.zone.is(ZoneKind::Battlefield);
    let start = || {
        let mut chars = match &obj.copy {
            Some(c) => initial(cards, card, Some(c)),
            None => *printed(cards, card, false),
        };
        chars.spell = spell;
        if room {
            doors(cards, card, obj.marks.doors, obj.copy.is_some_and(|c| c.colors.is_some()), &mut chars);
        }
        chars
    };

    // An object that has left the battlefield is as it last was there (CR 608.2h, 603.10).
    if obj.became.is_some()
        && let Some(last) = state.last_known(id)
    {
        let mut chars = start();
        chars.types = last.types;
        chars.subtypes = last.subtypes;
        chars.colors = last.colors;
        chars.power = last.power;
        chars.toughness = last.toughness;
        chars.keywords = last.keywords;
        chars.abilities = last.abilities;
        return chars;
    }

    let on_battlefield = obj.zone.is(ZoneKind::Battlefield) && obj.became.is_none();
    // What could apply at all, found once: most objects have no effects, no statics touch them, and no counters.
    // A spell's text can be changed too (CR 612.1: "target spell or permanent").
    let has_effects = (on_battlefield || spell) && state.effects.iter().any(|e| e.affected == id);
    let has_counters = on_battlefield && obj.counters.iter().any(|&c| c != 0);
    let has_own = !def.statics.is_empty();
    // The permanents (and graveyard cards) whose static abilities could apply, found once for every layer: most games
    // look at a flag and find none.
    // `statics_seen` is set by hone counters as well (CR 122.1j), which need no card with a static ability.
    let found = state.statics_seen.then(|| static_sources(cards, state));
    let sources: &[(ObjectId, ZoneKind, bool)] = found.as_ref().map_or(&[], |f| &f.list);
    // The permanents with hone counters attached to it, if any are.
    let honed: &[ObjectId] = found.as_ref().map_or(&[], |f| &f.honed);
    // The permanents given a static ability that affects other objects.
    let others: &[ObjectId] = found.as_ref().map_or(&[], |f| &f.granted);
    let has_hone = on_battlefield && honed.iter().any(|&e| state.obj(e).attached_to == Some(id));
    if !has_effects && !has_counters && !has_own && sources.is_empty() && others.is_empty() && !has_hone {
        if obj.copy.is_none() && !spell && !room {
            // Nothing acts on it: the card's own values, made once.
            return *printed(cards, card, stop > Layer::Ability);
        }
        let mut chars = start();
        if stop > Layer::Ability && obj.copy.is_none() && !room {
            let mut full = *printed(cards, card, true);
            full.spell = spell;
            return full;
        }
        if stop > Layer::Ability && chars.is(CardType::Land) {
            intrinsic_mana(cards, def, &mut chars);
        }
        if stop > Layer::Ability && chars.is(CardType::Battle) {
            intrinsic_siege(cards, &mut chars);
        }
        if stop > Layer::Ability {
            intrinsic_keyword_abilities(cards, &mut chars);
        }
        return chars;
    }
    let mut chars = start();
    let colors = obj.copy.and_then(|c| c.colors);
    let pt = obj.copy.and_then(|c| c.pt);
    // The continuous effects on it, oldest first.
    let mut fx: Vec<ContinuousEffect> =
        if has_effects { state.effects.iter().filter(|e| e.affected == id).copied().collect() } else { Vec::new() };
    // CR 122.1b, 613.1f, 613.7c: the ability a keyword counter gives applies in layer 6 at the timestamp of its counters, in
    // timestamp order with the effects there, so "loses all abilities" takes it from counters put on before it and not
    // from those put on after.
    if has_counters {
        keyword_counters(state, id, obj, &mut fx);
    }
    // The static abilities effects have granted it that it still has once layer 6 is done, each with its timestamp.
    let mut granted: Vec<(u32, &'static StaticDef)> = Vec::new();
    for layer in LAYERS {
        if layer >= stop {
            break;
        }
        if layer == Layer::PtDefine && has_effects {
            granted = granted_statics(cards, obj.timestamp, &fx, &chars);
        }
        // CR 305.6: the abilities a basic land type carries follow from the types layer 4 settled, as the object's
        // own from the start of layer 6.
        if layer == Layer::Ability && chars.is(CardType::Land) {
            intrinsic_mana(cards, def, &mut chars);
        }
        if layer == Layer::Ability && chars.is(CardType::Battle) {
            intrinsic_siege(cards, &mut chars);
        }
        if has_counters && layer == Layer::PtModify {
            counters(obj, &mut chars);
        }
        if has_hone && layer == Layer::PtModify {
            hone(cards, state, log, id, honed, &mut chars);
        }
        // CR 613.7, 613.8: within a layer, static abilities and other continuous effects apply in timestamp order,
        // interleaved, unless one depends on another.
        let givers = (sources, others);
        statics(cards, state, log, id, def, (colors.is_some(), pt.is_some()), givers, on_battlefield, layer, &fx, &granted, &mut chars);
        if layer == Layer::Ability {
            intrinsic_keyword_abilities(cards, &mut chars);
        }
    }
    chars
}

/// Each kind of keyword counter on the object (CR 122.1b) as the layer-6 effect it is, "gains [keyword]", at the timestamp
/// of its counters (CR 613.7c), put among the effects in timestamp order.
fn keyword_counters(state: &GameState, id: ObjectId, obj: &Obj, fx: &mut Vec<ContinuousEffect>) {
    for kind in CounterKind::KEYWORDS {
        if obj.counters(kind) > 0
            && let Some(k) = kind.keyword()
        {
            let timestamp = state.counter_stamp(id, kind);
            let at = fx.partition_point(|e| e.timestamp <= timestamp);
            fx.insert(
                at,
                ContinuousEffect {
                    source: id,
                    controller: obj.controller,
                    timestamp,
                    affected: id,
                    kind: EffectKind::AddKeywords(Keywords::of(&[k])),
                    duration: Duration::Indefinite,
                },
            );
        }
    }
}

/// The abilities the rules give an object for a keyword it has once layer 6 is done (CR 702.147a): decayed's triggered
/// ability, whether the keyword is printed, granted or a counter's. Its static half ("this creature can't block") is read
/// off the keyword by the blocking rule.
fn intrinsic_keyword_abilities(cards: &CardTable, chars: &mut Chars) {
    if chars.keywords.has(Keyword::Decayed)
        && let Some(slot) = chars.abilities.iter().position(|a| a.is_none())
    {
        chars.abilities[slot] = Some(AbilityRef { card: cards.intrinsic, index: cantrip_api::intrinsic::DECAYED });
    }
}

/// The static abilities effects have given the object (CR 613.1f) that it has once layer 6 is done, oldest first, each with the
/// timestamp its effect has (CR 613.7a): the later of the object's own and that of the effect that granted the ability.
/// "Loses all abilities" (CR 613.1f) at a later timestamp takes the ones granted before it, and a static ability that took
/// abilities away has taken them from `chars` already: only what the object still has is kept. A grant of the same ability
/// twice is two effects, each with its own timestamp. Guarded on the object's own effects, so an object nothing granted
/// anything to costs nothing here.
fn granted_statics(cards: &CardTable, object_timestamp: u32, fx: &[ContinuousEffect], chars: &Chars) -> Vec<(u32, &'static StaticDef)> {
    let mut granted: Vec<(u32, AbilityRef)> = Vec::new();
    for e in fx {
        match e.kind {
            // Its own: one that affects other objects applies from it as a source (`granted_to_others`).
            EffectKind::AddAbility(ability) if matches!(cards.ability(ability), AbilityDef::Static(s) if s.own) => {
                granted.push((e.timestamp.max(object_timestamp), ability));
            }
            EffectKind::LoseAllAbilities => granted.clear(),
            _ => {}
        }
    }
    granted
        .into_iter()
        .filter(|(_, ability)| chars.abilities.contains(&Some(*ability)))
        .filter_map(|(timestamp, ability)| match cards.ability(ability) {
            AbilityDef::Static(s) => Some((timestamp, s)),
            _ => None,
        })
        .collect()
}

/// The permanents (and graveyard cards) whose static abilities could apply to anything, for the state's current epoch:
/// found once per position, whatever the number of questions asked of it.
fn static_sources<'a>(cards: &CardTable, state: &'a GameState) -> std::cell::Ref<'a, cantrip_state::StaticSources> {
    {
        let have = state.static_sources.borrow();
        if have.epoch == state.epoch {
            return have;
        }
    }
    {
        let mut made = state.static_sources.borrow_mut();
        made.list.clear();
        made.strippers.clear();
        made.honed.clear();
        // A permanent that has lost all its abilities to an effect keeps the effects its static abilities had applied in
        // layers 1 to 5 by the time layer 6 took them (CR 613.6, Humility with Magus of the Moon); from layer 6 on it has
        // none (CR 613.1f, 613.8a).
        let note = |list: &mut Vec<(ObjectId, ZoneKind, bool)>, source: ObjectId, from: ZoneKind| {
            let sobj = state.obj(source);
            let scard = sobj.copy.map(|c| c.card).unwrap_or_else(|| cards.face(sobj.card, sobj.transformed));
            if cards.statics[scard.0 as usize] && list.len() < MAX_STATIC_SOURCES && live(state, source) {
                list.push((source, from, state.lost_all_abilities_to_an_effect(source)));
            }
        };
        for &source in state.battlefield() {
            note(&mut made.list, source, ZoneKind::Battlefield);
            // CR 122.1j: hone counters act on what their permanent is attached to, abilities or none.
            if state.obj(source).counters(CounterKind::Hone) > 0 && live(state, source) {
                made.honed.push(source);
            }
        }
        // An emblem's static abilities function in the command zone (CR 114.4).
        for &source in state.zone(Zone::COMMAND) {
            note(&mut made.list, source, ZoneKind::Battlefield);
        }
        if cards.graveyard_statics {
            for p in state.seats() {
                for &source in state.zone(Zone::graveyard(p)) {
                    note(&mut made.list, source, ZoneKind::Graveyard);
                }
            }
        }
        // A card encoded on a creature functions in exile (CR 702.99a).
        if cards.exile_statics {
            for p in state.seats() {
                for &source in state.zone(Zone::exile(p)) {
                    note(&mut made.list, source, ZoneKind::Exile);
                }
            }
        }
        // CR 613.7: static abilities apply in the timestamp order of the objects that have them.
        made.list.sort_by_key(|&(source, _, _)| state.obj(source).timestamp);
        let strippers: Vec<(ObjectId, ZoneKind)> = made
            .list
            .iter()
            .filter(|&&(source, from, lost)| !lost && cards.get(cards.current(state.obj(source))).statics.iter().any(|s| s.loses_all && s.from == from))
            .map(|&(source, from, _)| (source, from))
            .collect();
        made.strippers = strippers;
        made.granted.clear();
        for e in &state.effects {
            if let EffectKind::AddAbility(ability) = e.kind
                && matches!(cards.ability(ability), AbilityDef::Static(s) if !s.own)
                && live(state, e.affected)
                && state.obj(e.affected).zone.is(ZoneKind::Battlefield)
                && !made.granted.contains(&e.affected)
            {
                made.granted.push(e.affected);
            }
        }
        made.epoch = state.epoch;
    }
    state.static_sources.borrow()
}

/// The copiable values of `card` as an object has them before any layer applies: the card's own, or a copy effect's
/// (`copy`), with the abilities the card prints.
fn initial(cards: &CardTable, card: CardId, copy: Option<&Copiable>) -> Chars {
    let def = cards.get(card);
    let (extra_types, extra_subtypes, extra_abilities, extra_keywords, colors, pt, no_cost, copy_name, words) = match copy {
        Some(c) => (c.extra_types, c.extra_subtypes, c.extra_abilities, c.extra_keywords, c.colors, c.pt, c.no_cost, c.name, c.words),
        None => (TypeSet::NONE, SubtypeSet::NONE, [None; 4], Keywords::NONE, None, None, false, None, Words::NONE),
    };
    // "Except it's legendary" (CR 707.9b).
    let legendary = copy.is_some_and(|c| c.legendary);
    let mut abilities = [None; MAX_ABILITIES];
    let mut n = 0;
    // A dungeon or the rules' own card has more abilities than any object does; nothing reads them as an object's.
    for index in 0..def.abilities.len().min(MAX_ABILITIES) {
        abilities[n] = Some(AbilityRef { card, index: index as u8 });
        n += 1;
    }
    for extra in extra_abilities.into_iter().flatten() {
        abilities[n] = Some(extra);
        n += 1;
    }
    let mut chars = Chars {
        card,
        name: copy_name.unwrap_or_else(|| renamed(def, words)),
        types: TypeSet(def.types.0 | extra_types.0),
        supertypes: Supertypes { legendary: def.supertypes.legendary || legendary, ..def.supertypes },
        // CR 612.4: a token's subtypes are what the text that made it named.
        subtypes: words.subtypes(def.subtypes).union(extra_subtypes),
        // CR 707.9b: a copy that says what it is gets that, not what the card it copies is (707.9d). A token's color
        // words are its text's (CR 612.4).
        colors: colors.unwrap_or_else(|| words.colors(def.colors())),
        power: pt.map(|(p, _)| p).or(def.power),
        toughness: pt.map(|(_, t)| t).or(def.toughness),
        keywords: Keywords(def.keywords.0 | extra_keywords.0),
        protection: ColorSet::NONE,
        protection_colorless: false,
        protection_players: 0,
        protection_types: TypeSet::NONE,
        spell: false,
        protection_colored_spells: false,
        protection_everything: false,
        // CR 612.4: a color word of a token's text is the text's that made it.
        hexproof_from: worded_hexproof(def.hexproof_from, words),
        must_attack: false,
        abilities,
        no_cost,
        name_set: false,
    };
    changeling(&mut chars);
    chars
}

/// Printed "hexproof from [quality]" with the color words of its text as `words` has them (CR 612.2).
fn worded_hexproof(printed: HexproofFrom, words: Words) -> HexproofFrom {
    if printed.colors().is_empty() || words.is_printed() {
        return printed;
    }
    printed.with_colors(words.colors(printed.colors()))
}

/// Changeling (CR 702.73a): "This object is every creature type", a characteristic-defining ability that works in every
/// zone (CR 604.3) and applies first in layer 4 (CR 613.3). It gives only what the object can have: creature types
/// belong to creatures and kindreds (CR 205.3d, 205.3m). Only the printed ability is a characteristic-defining one
/// (CR 604.3a), so one granted in layer 6 gives nothing. A non-creature, non-Kindred object with Changeling that later
/// becomes a creature is not given the types: no printed card can reach that case (Firdoch Core is a Kindred).
fn changeling(chars: &mut Chars) {
    if chars.keywords.has(Keyword::Changeling) && (chars.is(CardType::Creature) || chars.is(CardType::Kindred)) {
        chars.subtypes = chars.subtypes.union(SubtypeSet::CREATURE_TYPES);
    }
}

/// What a Room's unlocked designations leave of it (CR 709.5): "as long as this permanent doesn't have the 'left half
/// unlocked' designation, it doesn't have the name, mana cost, or rules text of this object's left half", and the same of
/// the right. The name is that of the unlocked doors (none, one, or both), its colors those of their mana costs unless a
/// copy effect says otherwise (CR 707.9b), its abilities the unlocked doors', then any a copy effect added, and last the
/// unlock special action (CR 709.5e) of each door still locked.
fn doors(cards: &CardTable, card: CardId, unlocked: u8, colors_stated: bool, chars: &mut Chars) {
    let def = cards.get(card);
    let (left, right) = def.split.expect("a Room has two doors");
    let (left_id, right_id) = cards.halves[card.0 as usize].expect("a Room's doors are in the card table");
    chars.name = match unlocked & (DOOR_LEFT | DOOR_RIGHT) {
        0 => "",
        DOOR_LEFT => left.name,
        DOOR_RIGHT => right.name,
        _ => def.name,
    };
    if !colors_stated {
        chars.colors = def.room_cost(unlocked).colors();
    }
    let mut abilities = [None; MAX_ABILITIES];
    let mut n = 0;
    let mut push = |ability: AbilityRef| {
        if n < MAX_ABILITIES {
            abilities[n] = Some(ability);
            n += 1;
        }
    };
    for (door, half, id) in [(DOOR_LEFT, left, left_id), (DOOR_RIGHT, right, right_id)] {
        if unlocked & door != 0 {
            for index in 0..half.abilities.len() {
                push(AbilityRef { card: id, index: index as u8 });
            }
        }
    }
    for ability in chars.abilities.into_iter().flatten() {
        push(ability);
    }
    for (door, index) in [(DOOR_LEFT, cantrip_api::intrinsic::UNLOCK_LEFT), (DOOR_RIGHT, cantrip_api::intrinsic::UNLOCK_RIGHT)] {
        if unlocked & door == 0 {
            push(AbilityRef { card: cards.intrinsic, index });
        }
    }
    chars.abilities = abilities;
}

/// A token's name under changed text (CR 612.2a): a token named for its creature type ("Spirit") is named for the type
/// the text names now.
fn renamed(def: &CardDef, words: Words) -> &'static str {
    if words.is_printed() {
        return def.name;
    }
    match Subtype::named(def.name) {
        Some(s) if s.is_creature_type() && def.subtypes.has(s) => words.subtype(s).name(),
        _ => def.name,
    }
}

/// A card's characteristics with no copy effect and nothing acting on it, made once per card (`CardTable::printed`):
/// with `rules`, the abilities the rules give a land by its basic land types and a Siege (layer 6) too. `spell` is not
/// set: it is the object's, not the card's.
fn printed(cards: &CardTable, card: CardId, rules: bool) -> &Chars {
    cards.printed[card.0 as usize][rules as usize].get_or_init(|| {
        let def = cards.get(card);
        let mut chars = initial(cards, card, None);
        if rules {
            if chars.is(CardType::Land) {
                intrinsic_mana(cards, def, &mut chars);
            }
            if chars.is(CardType::Battle) {
                intrinsic_siege(cards, &mut chars);
            }
            intrinsic_keyword_abilities(cards, &mut chars);
        }
        Box::new(chars)
    })
}

/// Applies the continuous effects of `layer` to `id` (CR 613.1): its own characteristic-defining abilities first (CR
/// 613.3), then the effects of resolved spells and abilities, the static abilities effects granted it, those of
/// permanents (and cards that function elsewhere) and those effects granted permanents to give others, in timestamp order
/// (CR 613.7) unless one depends on another (CR 613.8, `by_dependency`).
fn statics(
    cards: &CardTable,
    state: &GameState,
    log: &[Record],
    id: ObjectId,
    def: &CardDef,
    overrides: (bool, bool),
    (sources, others): (&[(ObjectId, ZoneKind, bool)], &[ObjectId]),
    on_battlefield: bool,
    layer: Layer,
    fx: &[ContinuousEffect],
    granted: &[(u32, &'static StaticDef)],
    chars: &mut Chars,
) {
    let view = View { cards, state, log, stop: layer };
    // From layer 6 on, the permanents a "loses all abilities" static ability has taken everything from (CR 613.1f).
    let stripped = if layer >= Layer::Ability { lost_to_statics(cards, state, log) } else { Stripped::default() };
    // CR 707.9d: a copy that states its colors or its power and toughness does not copy a characteristic-defining
    // ability that would define them.
    let overridden = (layer == Layer::Color && overrides.0) || (layer == Layer::PtDefine && overrides.1);
    // CR 604.3, 613.1f: its own characteristic-defining abilities are static abilities, so layer 6 takes them away, and the layers
    // after it (a power and toughness one, 613.4a) find them gone. Those of layers 4 and 5 were applied before, and stay (CR 613.6).
    // CR 613.8a(c): one never depends on an effect that is not from a characteristic-defining ability, nor that on it.
    for st in def.statics.iter().filter(|s| s.own && s.layer == layer) {
        if overridden {
            continue;
        }
        if layer >= Layer::Ability && (state.lost_all_abilities_to_an_effect(id) || stripped.lost.contains(&id)) {
            if let Some((power, toughness)) = st.lost_pt {
                chars.power = Some(power);
                chars.toughness = Some(toughness);
            }
        } else {
            (st.apply)(&view, id, chars);
        }
    }
    // A card outside the game is affected by nothing but its own characteristic-defining abilities (CR 400.11c).
    if state.obj(id).zone.is(ZoneKind::Sideboard) {
        return;
    }
    // The static abilities effects gave permanents that affect other objects, in their timestamp order (CR 613.7a).
    let mut given: Vec<(u32, ObjectId, &'static StaticDef)> =
        if on_battlefield { others.iter().flat_map(|&holder| granted_to_others(cards, state, log, holder, layer)).collect() } else { Vec::new() };
    given.sort_by_key(|&(timestamp, _, _)| timestamp);
    let merge = Merge::new(cards, state, layer, on_battlefield, fx, granted, &given, sources, &stripped);
    in_order(&view, id, merge, chars);
}

thread_local! {
    /// Lists of a layer's entries to reuse: working out one object's layer asks for others' (`View::chars`), each with its
    /// own list, so they are taken and given back rather than shared.
    static ENTRIES: std::cell::RefCell<Vec<Vec<Entry>>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Applies a layer's effects (`merge`) to the object in timestamp order (CR 613.7), watching for a dependency (CR 613.8a):
/// each time an effect does something to the object, the earlier effects whose reach can change are asked again whether they
/// apply to it. If one's answer changed, it depends on the later effect, and the layer is worked out again from the start
/// by the dependency system (`by_dependency`). A layer with no effect whose reach can change before one that does
/// something costs what timestamp order costs; an anthem with the pumps after it, a look at the anthem after each.
fn in_order(view: &View, id: ObjectId, mut merge: Merge, chars: &mut Chars) {
    // None, or one: nothing to order.
    let Some(first) = merge.next() else { return };
    let Some(second) = merge.next() else {
        first.apply(view, id, chars);
        return;
    };
    let mut entries = ENTRIES.with(|pool| pool.borrow_mut().pop()).unwrap_or_default();
    entries.clear();
    entries.push(first);
    entries.push(second);
    while let Some(e) = merge.next() {
        entries.push(e);
    }
    // Nothing whose reach can change: timestamp order.
    if !entries.iter().any(Entry::conditional) {
        for entry in &entries {
            entry.apply(view, id, chars);
        }
        ENTRIES.with(|pool| pool.borrow_mut().push(entries));
        return;
    }
    let before = *chars;
    // Bit `k` of `conditional`: the `k`th entry's reach can change; of `reached`: it applied when its turn came.
    let mut conditional = 0u64;
    let mut reached = 0u64;
    let depends = 'apply: {
        for (k, entry) in entries.iter().enumerate() {
            if k == 64 {
                break 'apply true;
            }
            let reach = entry.reach(view, id, chars);
            if entry.conditional() {
                conditional |= 1 << k;
                reached |= u64::from(reach) << k;
            }
            if reach {
                entry.apply_reached(view, id, chars);
                // The earlier effects whose reach can change, asked again.
                let mut earlier = conditional & ((1 << k) - 1);
                while earlier != 0 {
                    let a = earlier.trailing_zeros() as usize;
                    earlier &= earlier - 1;
                    if entries[a].reach(view, id, chars) != (reached & (1 << a) != 0) {
                        break 'apply true;
                    }
                }
            }
        }
        false
    };
    if depends {
        *chars = before;
        by_dependency(view, id, entries.clone(), chars);
    }
    ENTRIES.with(|pool| pool.borrow_mut().push(entries));
}

/// One continuous effect of a layer that may apply to the object being worked out.
#[derive(Clone, Copy)]
enum Entry {
    /// A resolved spell's or ability's (CR 611.2), which names the object.
    Effect(ContinuousEffect),
    /// A static ability an effect granted the object that applies to it alone (CR 613.7a), with it as its source.
    Granted(&'static StaticDef),
    /// A static ability of `ObjectId` (its own, or one an effect granted it) that applies to the objects it says.
    Static(ObjectId, &'static StaticDef),
}

impl Entry {
    /// Whether another effect could change what it applies to or what it does (CR 613.8a(b)), which is all the dependency
    /// system judges: a static ability's reach ("other creatures", "each land"), and the resolved effects that do
    /// something only to an object that has something - a text change, only where its word is (CR 612.1); "all creature
    /// types", only to a creature or Kindred (CR 205.3d). Every other resolved effect names the objects it applies to
    /// (CR 611.2c) and does the same to each whatever was done before it, so nothing changes what it applies to or does.
    /// A change to an effect's text or existence is not judged here: the only existence the layers can take away is a
    /// static ability's, to "loses all abilities", which `lost_to_statics` orders (CR 613.8a), and no text-changing effect
    /// the engine has changes another effect's words.
    fn conditional(&self) -> bool {
        match self {
            Entry::Static(..) => true,
            Entry::Granted(_) => false,
            Entry::Effect(e) => matches!(e.kind, EffectKind::ChangeText { .. } | EffectKind::AddAllCreatureTypes),
        }
    }

    /// Whether it applies to the object as `chars` is now, and so does something to it (CR 613.8a(b)).
    fn reach(&self, view: &View, id: ObjectId, chars: &Chars) -> bool {
        match *self {
            Entry::Static(source, st) => (st.affects)(view, source, id, chars),
            Entry::Granted(_) => true,
            Entry::Effect(e) => match e.kind {
                EffectKind::ChangeText { from: Word::Subtype(from), .. } => {
                    chars.subtypes.has(from) && !(from.is_creature_type() && chars.keywords.has(Keyword::Changeling))
                }
                EffectKind::ChangeText { from: Word::Color(from), .. } => chars.hexproof_from.colors().has(from),
                EffectKind::AddAllCreatureTypes => chars.is(CardType::Creature) || chars.is(CardType::Kindred),
                _ => true,
            },
        }
    }

    /// Applies it to the object, which `reach` has said it applies to.
    fn apply_reached(&self, view: &View, id: ObjectId, chars: &mut Chars) {
        match *self {
            Entry::Effect(e) => apply_effect(&e, view.stop, chars),
            Entry::Granted(st) => (st.apply)(view, id, chars),
            Entry::Static(source, st) => (st.apply)(view, source, chars),
        }
    }

    fn apply(&self, view: &View, id: ObjectId, chars: &mut Chars) {
        match *self {
            Entry::Effect(e) => apply_effect(&e, view.stop, chars),
            Entry::Granted(st) => (st.apply)(view, id, chars),
            Entry::Static(source, st) => {
                if (st.affects)(view, source, id, chars) {
                    (st.apply)(view, source, chars);
                }
            }
        }
    }
}

/// The effects of one layer that may apply to the object, in timestamp order (CR 613.7): four sequences, each in its own
/// order, merged. At one timestamp a permanent's static abilities come first, then those effects granted others, then
/// resolved effects, then abilities granted the object itself (CR 613.7n: an object's own static ability is earlier than
/// an effect created with it).
#[derive(Clone)]
struct Merge<'a> {
    cards: &'a CardTable,
    state: &'a GameState,
    layer: Layer,
    on_battlefield: bool,
    fx: &'a [ContinuousEffect],
    granted: &'a [(u32, &'static StaticDef)],
    given: &'a [(u32, ObjectId, &'static StaticDef)],
    sources: &'a [(ObjectId, ZoneKind, bool)],
    /// From layer 6 on, the permanents a "loses all abilities" static ability has stripped, and those whose such
    /// abilities apply.
    stripped: &'a Stripped,
    /// The next of each: source, given, effect, granted.
    next: [usize; 4],
    /// The current source's static abilities (none once it has lost them), and the next of them.
    statics: Option<&'static [StaticDef]>,
    within: usize,
    /// Each sequence's next entry's timestamp, worked out once it was last moved past (`settled`).
    head: [Option<u32>; 4],
    settled: [bool; 4],
}

impl<'a> Merge<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        cards: &'a CardTable,
        state: &'a GameState,
        layer: Layer,
        on_battlefield: bool,
        fx: &'a [ContinuousEffect],
        granted: &'a [(u32, &'static StaticDef)],
        given: &'a [(u32, ObjectId, &'static StaticDef)],
        sources: &'a [(ObjectId, ZoneKind, bool)],
        stripped: &'a Stripped,
    ) -> Merge<'a> {
        Merge {
            cards,
            state,
            layer,
            on_battlefield,
            fx,
            granted,
            given,
            sources,
            stripped,
            next: [0; 4],
            statics: None,
            within: 0,
            head: [None; 4],
            settled: [false; 4],
        }
    }

    /// Moves sequence `i` past what is not of this layer, and notes its next entry's timestamp.
    fn settle(&mut self, i: usize) {
        let layer = self.layer;
        self.head[i] = match i {
            0 => {
                let mut found = None;
                while let Some(&(source, from, lost)) = self.sources.get(self.next[0]) {
                    let statics = match self.statics {
                        Some(statics) => statics,
                        None => {
                            // CR 613.1f, 613.8a: a permanent that has lost all its abilities has no static abilities from
                            // layer 6 on. Before it, they have applied (CR 613.6), and so has the "loses all abilities" ability
                            // of one that took its own (Humility made a creature), which goes on applying (CR 613.6).
                            let gone = layer >= Layer::Ability
                                && (lost || self.stripped.lost.contains(&source))
                                && !self.stripped.applying.contains(&source);
                            let statics: &'static [StaticDef] = if gone { &[] } else { self.source_statics(source) };
                            self.statics = Some(statics);
                            statics
                        }
                    };
                    let on_battlefield = self.on_battlefield;
                    while statics
                        .get(self.within)
                        .is_some_and(|st| st.own || st.layer != layer || st.from != from || !(on_battlefield || st.everywhere))
                    {
                        self.within += 1;
                    }
                    if self.within < statics.len() {
                        found = Some(self.state.obj(source).timestamp);
                        break;
                    }
                    self.next[0] += 1;
                    self.within = 0;
                    self.statics = None;
                }
                found
            }
            1 => self.given.get(self.next[1]).map(|&(t, _, _)| t),
            2 => {
                while self.fx.get(self.next[2]).is_some_and(|e| effect_layer(e.kind) != Some(layer)) {
                    self.next[2] += 1;
                }
                self.fx.get(self.next[2]).map(|e| e.timestamp)
            }
            _ => {
                while self.granted.get(self.next[3]).is_some_and(|(_, st)| st.layer != layer) {
                    self.next[3] += 1;
                }
                self.granted.get(self.next[3]).map(|&(t, _)| t)
            }
        };
        self.settled[i] = true;
    }

    fn source_statics(&self, source: ObjectId) -> &'static [StaticDef] {
        let sobj = self.state.obj(source);
        let scard = sobj.copy.map(|c| c.card).unwrap_or_else(|| self.cards.face(sobj.card, sobj.transformed));
        self.cards.get(scard).statics
    }

    fn next(&mut self) -> Option<Entry> {
        let mut which: Option<usize> = None;
        for i in 0..4 {
            if !self.settled[i] {
                self.settle(i);
            }
            if let Some(t) = self.head[i]
                && which.is_none_or(|w| self.head[w].is_some_and(|best| t < best))
            {
                which = Some(i);
            }
        }
        let which = which?;
        self.settled[which] = false;
        Some(match which {
            0 => {
                let (source, _, _) = self.sources[self.next[0]];
                let st = &self.statics.unwrap_or(&[])[self.within];
                self.within += 1;
                Entry::Static(source, st)
            }
            1 => {
                let (_, holder, st) = self.given[self.next[1]];
                self.next[1] += 1;
                Entry::Static(holder, st)
            }
            2 => {
                self.next[2] += 1;
                Entry::Effect(self.fx[self.next[2] - 1])
            }
            _ => {
                self.next[3] += 1;
                Entry::Granted(self.granted[self.next[3] - 1].1)
            }
        })
    }
}

/// The dependency system (CR 613.8): `pending`, in timestamp order, applied one at a time. An effect depends on another of
/// the layer if applying the other would change what it applies to or what it does to it (613.8a(b), `Entry::reach`), and
/// waits until just after everything it depends on has applied; effects free to apply go in timestamp order, and so do the
/// effects of a dependency loop among themselves (613.8b). After each, what depends on what is worked out again (613.8c).
///
/// What depends on what is judged on this object, the one being worked out. Another effect's reach is the set of every
/// object it applies to, which this object's question cannot see; so an effect that would change what the first applies to
/// only once what it waits for has applied ("lands are creatures", waiting for the effect that makes this a land, before
/// "creatures are artifacts") counts as changing it, as it does for the lands already on the battlefield. A dependency
/// through another object alone is not seen (known issue KI-39).
fn by_dependency(view: &View, id: ObjectId, mut pending: Vec<Entry>, chars: &mut Chars) {
    while !pending.is_empty() {
        let i = ready(view, id, &pending, chars);
        let entry = pending.remove(i);
        entry.apply(view, id, chars);
    }
}

/// The earliest of `pending` that depends on nothing still to apply but what is in a loop with it (CR 613.8b).
fn ready(view: &View, id: ObjectId, pending: &[Entry], chars: &Chars) -> usize {
    let n = pending.len();
    // Whether `a` depends on `b` with the object as `before`: applying `b` changes whether `a` applies to it.
    let changes = |a: usize, b: usize, before: &Chars| {
        let mut after = *before;
        pending[b].apply(view, id, &mut after);
        pending[a].reach(view, id, before) != pending[a].reach(view, id, &after)
    };
    let mut direct: Vec<Vec<bool>> =
        (0..n).map(|a| (0..n).map(|b| a != b && pending[a].conditional() && changes(a, b, chars)).collect()).collect();
    // Most often nothing depends on anything: the earliest applies.
    if direct.iter().all(|row| row.iter().all(|&d| !d)) {
        return 0;
    }
    // Which each depends on through others (a loop is an effect that depends on itself this way).
    let closure = |direct: &Vec<Vec<bool>>| {
        let mut through = direct.clone();
        for k in 0..n {
            for a in 0..n {
                if through[a][k] {
                    for b in 0..n {
                        if through[k][b] {
                            through[a][b] = true;
                        }
                    }
                }
            }
        }
        through
    };
    let mut through = closure(&direct);
    // `b` that waits for others changes what `a` applies to if it would once they have applied.
    loop {
        let mut more = false;
        for a in (0..n).filter(|&a| pending[a].conditional()) {
            for b in 0..n {
                if b == a || direct[a][b] || !through[b].iter().any(|&w| w) {
                    continue;
                }
                let mut first = *chars;
                for (c, entry) in pending.iter().enumerate() {
                    if through[b][c] && c != a && c != b {
                        entry.apply(view, id, &mut first);
                    }
                }
                if changes(a, b, &first) {
                    direct[a][b] = true;
                    more = true;
                }
            }
        }
        if !more {
            break;
        }
        through = closure(&direct);
    }
    // A loop has a member that waits on nothing outside it, so one is always ready.
    (0..n).find(|&a| (0..n).all(|b| !direct[a][b] || through[b][a])).unwrap_or(0)
}

/// The layer a resolved effect of this kind applies in (CR 613.1), as `apply_effect` applies it; none for an effect that
/// is not a characteristic's.
fn effect_layer(kind: EffectKind) -> Option<Layer> {
    Some(match kind {
        EffectKind::ChangeText { .. } | EffectKind::SetName(_) => Layer::Text,
        EffectKind::AddTypes(_)
        | EffectKind::AddSubtype(_)
        | EffectKind::LoseAllCreatureTypes
        | EffectKind::AddAllCreatureTypes
        | EffectKind::SetCreatureType(_)
        | EffectKind::SetTypes(_) => Layer::Type,
        EffectKind::SetColors(_) => Layer::Color,
        EffectKind::LoseAllAbilities
        | EffectKind::Protection(_)
        | EffectKind::ProtectionFromColorless
        | EffectKind::ProtectionFromTypes(_)
        | EffectKind::ProtectionFromEverything
        | EffectKind::ProtectionFromPlayer(_)
        | EffectKind::AddKeywords(_)
        | EffectKind::Suspected
        | EffectKind::LoseKeywords(_)
        | EffectKind::AddAbility(_)
        | EffectKind::MustAttack
        | EffectKind::Goaded
        | EffectKind::MustAttackPlayer(_) => Layer::Ability,
        EffectKind::SetPt(..) => Layer::PtSet,
        EffectKind::Pump(..) => Layer::PtModify,
        EffectKind::SwitchPt => Layer::PtSwitch,
        _ => return None,
    })
}

/// The static abilities of `layer` effects have given `holder` that affect other objects (CR 613.1f), each with its
/// timestamp (CR 613.7a: the later of the holder's and the granting effect's), that it still has once its abilities are
/// settled: a later "loses all abilities" takes them, as a static ability that removes abilities does.
fn granted_to_others(cards: &CardTable, state: &GameState, log: &[Record], holder: ObjectId, layer: Layer) -> Vec<(u32, ObjectId, &'static StaticDef)> {
    // Whether it has the ability is settled by layer 6, so only abilities of the layers after it can be told apart without
    // asking the layers of the very object being worked out (`granted_static_ability`).
    if layer < Layer::PtDefine {
        return Vec::new();
    }
    let object_timestamp = state.obj(holder).timestamp;
    let mut found: Vec<(u32, AbilityRef, &'static StaticDef)> = Vec::new();
    for e in state.effects.iter().filter(|e| e.affected == holder) {
        match e.kind {
            EffectKind::AddAbility(ability) => {
                if let AbilityDef::Static(s) = cards.ability(ability)
                    && !s.own
                {
                    found.push((e.timestamp.max(object_timestamp), ability, s));
                }
            }
            EffectKind::LoseAllAbilities => found.clear(),
            _ => {}
        }
    }
    found.retain(|(_, _, s)| s.layer == layer);
    if found.is_empty() {
        return Vec::new();
    }
    let has = chars_before(cards, state, log, holder, Layer::PtDefine);
    found.into_iter().filter(|(_, ability, _)| has.abilities.contains(&Some(*ability))).map(|(t, _, s)| (t, holder, s)).collect()
}

/// A continuous effect a resolved spell or ability created (CR 611.2), applied if it is in `layer`.
fn apply_effect(effect: &ContinuousEffect, layer: Layer, chars: &mut Chars) {
    {
        match (layer, effect.kind) {
            // CR 612.1: a subtype in the type line is a word of its text; the rules text's words are read by card code.
            (Layer::Text, EffectKind::ChangeText { from: Word::Subtype(from), to: Word::Subtype(to) }) => {
                // Changeling's creature types stay: its ability applies after the layer-3 change.
                if chars.subtypes.has(from) && !(from.is_creature_type() && chars.keywords.has(Keyword::Changeling)) {
                    chars.subtypes = chars.subtypes.without(from).with(to);
                }
            }
            // CR 612.8: an effect that sets the name is a text-changing effect; the object loses every name it had.
            (Layer::Text, EffectKind::SetName(name)) => {
                chars.name = name;
                chars.name_set = true;
            }
            // CR 612.2: a color word of a "hexproof from [color]" ability is a word of the text; only the printed abilities
            // are there yet (what layer 6 grants comes after).
            (Layer::Text, EffectKind::ChangeText { from: Word::Color(from), to: Word::Color(to) }) => {
                let colors = chars.hexproof_from.colors();
                if colors.has(from) {
                    chars.hexproof_from = chars.hexproof_from.with_colors(ColorSet(colors.0 & !ColorSet::of(from).0).with(to));
                }
            }
            // CR 205.1b: added types are added; a land that becomes a creature is still a land.
            (Layer::Type, EffectKind::AddTypes(types)) => chars.types = TypeSet(chars.types.0 | types.0),
            (Layer::Type, EffectKind::AddSubtype(s)) => chars.subtypes = chars.subtypes.with(s),
            (Layer::Type, EffectKind::LoseAllCreatureTypes) => {
                chars.subtypes = chars.subtypes.iter().filter(|t| !t.is_creature_type()).fold(SubtypeSet::NONE, SubtypeSet::with);
            }
            // CR 205.3d: creature types belong to creatures and Kindreds.
            (Layer::Type, EffectKind::AddAllCreatureTypes) => {
                if chars.is(CardType::Creature) || chars.is(CardType::Kindred) {
                    chars.subtypes = chars.subtypes.union(SubtypeSet::CREATURE_TYPES);
                }
            }
            // CR 205.1a: a creature type that is set replaces every creature type, and no other subtype.
            (Layer::Type, EffectKind::SetCreatureType(s)) => {
                let kept = chars.subtypes.iter().filter(|t| !t.is_creature_type()).fold(SubtypeSet::NONE, SubtypeSet::with);
                chars.subtypes = kept.with(s);
            }
            // CR 613.1d: set types replace every card type, and the subtypes with them.
            (Layer::Type, EffectKind::SetTypes(types)) => {
                chars.types = types;
                chars.subtypes = SubtypeSet::NONE;
            }
            // CR 105.3: "becomes white and blue" is what it is now, not what it is as well.
            (Layer::Color, EffectKind::SetColors(colors)) => chars.colors = colors,
            (Layer::PtSet, EffectKind::SetPt(p, t)) => {
                chars.power = Some(p);
                chars.toughness = Some(t);
            }
            // CR 613.1f: every kind of ability goes, at this effect's timestamp: what is granted after it is kept.
            (Layer::Ability, EffectKind::LoseAllAbilities) => chars.lose_all_abilities(),
            (Layer::Ability, EffectKind::Protection(colors)) => chars.protection = ColorSet(chars.protection.0 | colors.0),
            (Layer::Ability, EffectKind::ProtectionFromColorless) => chars.protection_colorless = true,
            (Layer::Ability, EffectKind::ProtectionFromTypes(types)) => chars.protection_types = TypeSet(chars.protection_types.0 | types.0),
            (Layer::Ability, EffectKind::ProtectionFromEverything) => chars.protection_everything = true,
            (Layer::Ability, EffectKind::ProtectionFromPlayer(p)) => chars.protection_players |= 1 << p.0,
            (Layer::Ability, EffectKind::AddKeywords(k)) => chars.keywords = Keywords(chars.keywords.0 | k.0),
            // CR 701.60c: a suspected permanent has menace and "This creature can't block" for as long as it is suspected.
            (Layer::Ability, EffectKind::Suspected) => chars.keywords = chars.keywords.with(Keyword::Menace).with(Keyword::CantBlock),
            // CR 613.1f: lost at this effect's timestamp, so a keyword granted after it is kept. Losing hexproof loses every
            // "hexproof from [quality]" too (CR 702.11e).
            (Layer::Ability, EffectKind::LoseKeywords(k)) => {
                chars.keywords = Keywords(chars.keywords.0 & !k.0);
                if k.has(Keyword::Hexproof) {
                    chars.hexproof_from = HexproofFrom::NONE;
                }
            }
            (Layer::Ability, EffectKind::AddAbility(ability)) => {
                if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
                    chars.abilities[slot] = Some(ability);
                }
            }
            (Layer::Ability, EffectKind::MustAttack | EffectKind::Goaded | EffectKind::MustAttackPlayer(_)) => chars.must_attack = true,
            // CR 613.4d: power and toughness switch, whatever set and modified them first.
            (Layer::PtSwitch, EffectKind::SwitchPt) => std::mem::swap(&mut chars.power, &mut chars.toughness),
            (Layer::PtModify, EffectKind::Pump(p, t)) => {
                chars.power = chars.power.map(|v| v + p);
                chars.toughness = chars.toughness.map(|v| v + t);
            }
            _ => {}
        }
    }
}

/// +1/+1 and -1/-1 counters, in layer 7c (CR 613.4c). Keyword counters are layer 6's, among its effects (`keyword_counters`).
fn counters(obj: &Obj, chars: &mut Chars) {
    let plus = obj.counters(CounterKind::PlusOne) - obj.counters(CounterKind::MinusOne);
    if plus != 0 {
        chars.power = chars.power.map(|v| v + plus);
        chars.toughness = chars.toughness.map(|v| v + plus);
    }
}

/// Hone counters (CR 122.1j): "A hone counter on an Equipment gives +1/+0 to any creature that Equipment is attached to",
/// in layer 7c with the other counters (CR 613.4c). The effect is the counter's, not an ability's, so an Equipment that has
/// lost its abilities still gives it, and one that mentions no hone counter does too (the rulings). Each permanent with hone
/// counters attached to `id` gives one +1/+0 per counter while it is an Equipment; a creature gets nothing from a power it
/// lacks.
fn hone(cards: &CardTable, state: &GameState, log: &[Record], id: ObjectId, honed: &[ObjectId], chars: &mut Chars) {
    for &equipment in honed {
        let e = state.obj(equipment);
        // Its types are settled by layer 4, before any power is: what it is attached to cannot change them.
        if e.attached_to == Some(id) && chars_before(cards, state, log, equipment, Layer::Color).has_subtype(Subtype::Equipment) {
            let bonus = e.counters(CounterKind::Hone);
            chars.power = chars.power.map(|v| v + bonus);
        }
    }
}

/// A Siege's defeat ability (CR 310.12b): the rules give it to every Siege, after the abilities it prints.
fn intrinsic_siege(cards: &CardTable, chars: &mut Chars) {
    if chars.has_subtype(Subtype::Siege)
        && let Some(slot) = chars.abilities.iter().position(|a| a.is_none())
    {
        chars.abilities[slot] = Some(AbilityRef { card: cards.intrinsic, index: cantrip_api::intrinsic::SIEGE_DEFEAT });
    }
}

/// One mana ability per basic land type the land has (CR 305.6): after its printed mana abilities, its printed types
/// in printed order, then any an effect gave it, in WUBRG order (the C# engine's order).
fn intrinsic_mana(cards: &CardTable, def: &CardDef, chars: &mut Chars) {
    let basic = |m: Mana| match m {
        Mana::W => Subtype::Plains,
        Mana::U => Subtype::Island,
        Mana::B => Subtype::Swamp,
        Mana::R => Subtype::Mountain,
        _ => Subtype::Forest,
    };
    let mut order = [None; 5];
    let mut k = 0;
    for &m in def.basic_land_types {
        if chars.has_subtype(basic(m)) && !order.contains(&Some(m)) {
            order[k] = Some(m);
            k += 1;
        }
    }
    for m in [Mana::W, Mana::U, Mana::B, Mana::R, Mana::G] {
        if chars.has_subtype(basic(m)) && !order.contains(&Some(m)) {
            order[k] = Some(m);
            k += 1;
        }
    }
    if k == 0 {
        return;
    }
    if chars.abilities[0].is_none() {
        for (i, m) in order.into_iter().flatten().enumerate() {
            chars.abilities[i] = Some(AbilityRef { card: cards.intrinsic, index: m as u8 });
        }
        return;
    }
    // After the printed mana abilities, which come first (CR 605; the C# engine numbers them first).
    let at = chars
        .abilities
        .iter()
        .position(|a| !matches!(a, Some(r) if matches!(cards.ability(*r), AbilityDef::Activated(d) if d.is_mana_ability())))
        .unwrap_or(MAX_ABILITIES);
    let mut out = [None; MAX_ABILITIES];
    let mut n = 0;
    for a in chars.abilities[..at].iter().flatten() {
        out[n] = Some(*a);
        n += 1;
    }
    for m in order.into_iter().flatten() {
        if n < MAX_ABILITIES {
            out[n] = Some(AbilityRef { card: cards.intrinsic, index: m as u8 });
            n += 1;
        }
    }
    for a in chars.abilities[at..].iter().flatten() {
        if n < MAX_ABILITIES {
            out[n] = Some(*a);
            n += 1;
        }
    }
    chars.abilities = out;
}

/// Whether the object is still the object it was: it has not changed zones (CR 400.7) or ceased to exist.
pub fn live(state: &GameState, id: ObjectId) -> bool {
    let obj = state.obj(id);
    obj.became.is_none() && !obj.zone.is(ZoneKind::Gone)
}

/// The rules' half of target legality (CR 115.4, 702.11, 702.16): the target still exists where it was, and nothing
/// about it forbids `asker`'s source from targeting it.
pub fn can_target(cards: &CardTable, state: &GameState, log: &[Record], asker: &Asker, target: Target) -> bool {
    match target {
        Target::None => false,
        // An ability on the stack can be targeted while it is there (CR 115.1, 115.5).
        Target::Ability(serial) => state.stack.iter().any(|i| i.serial == serial && i.kind != StackKind::Spell),
        // A player with hexproof (CR 702.11c) can't be targeted by their opponents' spells and abilities.
        // A player with protection from everything (CR 702.16j) can't be targeted by anything, their own included.
        Target::Player(p) => {
            !state.player(p).lost
                && !(p != asker.controller
                    && state.effects.iter().any(|e| e.kind == EffectKind::PlayerHexproof && e.controller == p))
                && !state.effects.iter().any(|e| e.kind == EffectKind::PlayerProtectionFromEverything && e.controller == p)
        }
        Target::Object(id) => {
            if !live(state, id) {
                return false;
            }
            let obj = state.obj(id);
            if obj.zone.is(ZoneKind::Battlefield) {
                let target_chars = chars(cards, state, log, id);
                // Shroud: no player may target it (CR 702.18a); hexproof: no opponent (CR 702.11b).
                if target_chars.has(Keyword::Shroud) {
                    return false;
                }
                let opponent = obj.controller != asker.controller;
                if target_chars.has(Keyword::Hexproof) && opponent && !targeting_rule(cards, state, log, TargetingRuleKind::IgnoreHexproof, id) {
                    return false;
                }
                let mut source_chars = chars(cards, state, log, asker.source);
                // A card that is not on the battlefield or the stack and asks for a target is about to be cast: it
                // moves to the stack before its targets are chosen (CR 601.2a), so it is judged as the spell it
                // will be, which protection from spells (CR 702.16a) turns away.
                let source = state.obj(asker.source);
                if !source.zone.is(ZoneKind::Battlefield) && !source.zone.is(ZoneKind::Stack) {
                    source_chars.spell = true;
                }
                // Hexproof from [quality] (CR 702.11d): no [quality] spell an opponent controls, nor an ability an opponent
                // controls from a [quality] source, the source as it is now (a spell, or an ability's source on the
                // battlefield or as it last was). A hexproof ability, so what ignores hexproof ignores it (CR 702.11e).
                if opponent
                    && target_chars.hexproof_from.covers(source_chars.colors, source_chars.types)
                    && !targeting_rule(cards, state, log, TargetingRuleKind::IgnoreHexproof, id)
                {
                    return false;
                }
                if target_chars.protected_from_by(&source_chars, asker.controller) {
                    return false;
                }
            }
            true
        }
    }
}

/// Whether `id` has lost all its abilities (CR 613.1f): to a resolved effect, or to a static ability
/// (`StaticDef::loses_all`, which `lost_to_statics` works out). `Query::lost_all_abilities` is this.
pub fn lost_all_abilities(cards: &CardTable, state: &GameState, log: &[Record], id: ObjectId) -> bool {
    state.lost_all_abilities_to_an_effect(id) || (state.statics_seen && lost_to_statics(cards, state, log).lost.contains(&id))
}

/// What `lost_to_statics` found: the permanents that lost all their abilities to a static ability, and the sources whose
/// such abilities apply.
#[derive(Default)]
struct Stripped {
    lost: Vec<ObjectId>,
    applying: Vec<ObjectId>,
}

/// The permanents that have lost all their abilities to a "loses all abilities" static ability (CR 613.1f), found once
/// per position and history, and empty, at the cost of a look at the source list, while no such ability is on the
/// battlefield.
///
/// Each such ability applies in layer 6 to what its `affects` accepts, judged on the characteristics layers 1 to 5 made
/// (CR 613.1). One that would take the abilities of another such ability's source is applied first, since the other's
/// existence depends on it (CR 613.8a), and the abilities it takes no longer apply; where each would take the other's,
/// the earlier timestamp goes first (CR 613.8b). The source of one a resolved effect took away is left out already: its
/// ability depends on that effect, which comes first.
fn lost_to_statics(cards: &CardTable, state: &GameState, log: &[Record]) -> Stripped {
    if !state.statics_seen {
        return Stripped::default();
    }
    let strippers = static_sources(cards, state).strippers.clone();
    if strippers.is_empty() {
        return Stripped::default();
    }
    let key = (state.epoch, log.len());
    {
        let have = state.lost_to_statics.borrow();
        // An `affects` that asked for characteristics past layer 6 would come back here: it is answered as layer 6
        // began, before any ability was taken.
        if have.finding {
            return Stripped::default();
        }
        if have.key == key {
            return Stripped { lost: have.objects.clone(), applying: have.applying.clone() };
        }
    }
    state.lost_to_statics.borrow_mut().finding = true;
    let view = View { cards, state, log, stop: Layer::Ability };
    let strips = |(source, from): (ObjectId, ZoneKind), id: ObjectId| -> bool {
        let obj = state.obj(id);
        if !obj.zone.is(ZoneKind::Battlefield) || obj.became.is_some() {
            return false;
        }
        let so_far = chars_before(cards, state, log, id, Layer::Ability);
        cards.get(cards.current(state.obj(source))).statics.iter().any(|s| s.loses_all && s.from == from && (s.affects)(&view, source, id, &so_far))
    };
    let mut pending = strippers;
    let mut applied: Vec<(ObjectId, ZoneKind)> = Vec::new();
    while !pending.is_empty() {
        // The first, in timestamp order, that no other one still waiting would take away; failing that, the first.
        let next = (0..pending.len()).find(|&i| !pending.iter().any(|&other| other.0 != pending[i].0 && strips(other, pending[i].0))).unwrap_or(0);
        let ability = pending.remove(next);
        pending.retain(|&other| !strips(ability, other.0));
        applied.push(ability);
    }
    let objects: Vec<ObjectId> =
        state.battlefield().iter().copied().filter(|&id| applied.iter().any(|&ability| strips(ability, id))).collect();
    let applying: Vec<ObjectId> = applied.iter().map(|&(source, _)| source).collect();
    let mut have = state.lost_to_statics.borrow_mut();
    have.key = key;
    have.objects = objects.clone();
    have.applying = applying.clone();
    have.finding = false;
    Stripped { lost: objects, applying }
}

/// Whether a targeting rule of `kind` (CR 609.4, 702.11, 702.21) applies to `subject`: some permanent on the battlefield has a static
/// ability that names it. Found from the board, so asked only when it would change the answer (a hexproof target, a ward trigger).
pub fn targeting_rule(cards: &CardTable, state: &GameState, log: &[Record], kind: TargetingRuleKind, subject: ObjectId) -> bool {
    let view = View { cards, state, log, stop: Layer::Ability };
    state.battlefield().iter().any(|&source| {
        let obj = state.obj(source);
        let card = cards.current(obj);
        // A permanent that has lost all its abilities has no static abilities (CR 613.1f).
        cards.targeting_rules[card.0 as usize]
            && !lost_all_abilities(cards, state, log, source)
            && cards.get(card).targeting_rules.iter().any(|r| r.kind == kind && (r.applies)(&view, source, subject))
    })
}

/// Summoning sickness (CR 302.6): a creature that has not been under its controller's control continuously since
/// their most recent turn began, and has no haste.
pub fn summoning_sick(cards: &CardTable, state: &GameState, log: &[Record], id: ObjectId) -> bool {
    let obj = state.obj(id);
    let c = chars(cards, state, log, id);
    c.is(CardType::Creature) && !c.has(Keyword::Haste) && obj.controlled_since >= state.player(obj.controller).turn_began
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop(_: &dyn LayerQuery, _: ObjectId, _: &mut Chars) {}

    static HOLDER: CardDef = CardDef {
        name: "Holder",
        delayed: &[granted_static(Layer::PtSet, noop, "This creature's base power is 3.")],
        ..CardDef::BLANK
    };

    fn effect(timestamp: u32, kind: EffectKind) -> ContinuousEffect {
        ContinuousEffect {
            source: ObjectId(1),
            controller: PlayerId(0),
            timestamp,
            affected: ObjectId(1),
            kind,
            duration: Duration::EndOfTurn,
        }
    }

    fn stamps(granted: Vec<(u32, &'static StaticDef)>) -> Vec<u32> {
        granted.into_iter().map(|(t, _)| t).collect()
    }

    /// CR 613.1f, 613.7: "loses all abilities" takes the abilities the object has when it applies, so a grant with a LATER
    /// timestamp is kept, one with an EARLIER timestamp is lost, and an object stripped first and given the ability by a
    /// later effect has it (at the later effect's timestamp, CR 613.7a). No card can express the stripped-first case
    /// with Ms. Marvel: without her abilities she has no Embiggen Fist to make the grant.
    #[test]
    fn a_loses_all_abilities_applies_only_to_grants_before_it() {
        let cards = CardTable::new([&HOLDER]);
        let card = cards.find("Holder").unwrap();
        let ability = AbilityRef::delayed(card, 0);
        let mut abilities = [None; MAX_ABILITIES];
        abilities[0] = Some(ability);
        let chars = Chars {
            card,
            name: "Holder",
            types: TypeSet::NONE,
            supertypes: Supertypes::default(),
            subtypes: SubtypeSet::NONE,
            colors: ColorSet::NONE,
            power: Some(1),
            toughness: Some(4),
            keywords: Keywords::NONE,
            protection: ColorSet::NONE,
            protection_colorless: false,
            protection_players: 0,
            protection_types: TypeSet::NONE,
            spell: false,
            protection_colored_spells: false,
            protection_everything: false,
            hexproof_from: HexproofFrom::NONE,
            must_attack: false,
            abilities,
            no_cost: false,
            name_set: false,
        };
        let grant = |t| effect(t, EffectKind::AddAbility(ability));
        let strip = |t| effect(t, EffectKind::LoseAllAbilities);

        assert!(granted_statics(&cards, 1, &[grant(5), strip(7)], &chars).is_empty());
        assert_eq!(stamps(granted_statics(&cards, 1, &[strip(5), grant(7)], &chars)), [7]);
        assert_eq!(stamps(granted_statics(&cards, 1, &[grant(3), strip(5), grant(7)], &chars)), [7]);
        assert_eq!(stamps(granted_statics(&cards, 9, &[grant(7)], &chars)), [9]);
    }

    // --- The dependency system (CR 613.8), with static abilities and effects of layer 4 made for the purpose.

    fn creature(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
        c.is(CardType::Creature)
    }
    fn land(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
        c.is(CardType::Land)
    }
    fn not_artifact(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
        !c.is(CardType::Artifact)
    }
    fn not_enchantment(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
        !c.is(CardType::Enchantment)
    }
    fn add_artifact(_: &dyn LayerQuery, _: ObjectId, c: &mut Chars) {
        c.types = c.types.with(CardType::Artifact);
    }
    fn add_creature(_: &dyn LayerQuery, _: ObjectId, c: &mut Chars) {
        c.types = c.types.with(CardType::Creature);
    }
    fn add_enchantment(_: &dyn LayerQuery, _: ObjectId, c: &mut Chars) {
        c.types = c.types.with(CardType::Enchantment);
    }

    /// "Creatures are artifacts" (Ygra's shape).
    static CREATURES_ARE_ARTIFACTS: StaticDef = static_ability(Layer::Type, creature, add_artifact, "");
    /// "Lands are creatures" (Life and Limb's shape).
    static LANDS_ARE_CREATURES: StaticDef = static_ability(Layer::Type, land, add_creature, "");
    static NONARTIFACTS_ARE_ENCHANTMENTS: StaticDef = static_ability(Layer::Type, not_artifact, add_enchantment, "");
    static NONENCHANTMENTS_ARE_ARTIFACTS: StaticDef = static_ability(Layer::Type, not_enchantment, add_artifact, "");

    /// The types `pending` (in timestamp order) leave an object with no types, through the dependency system.
    fn types_after(pending: &[Entry], start: TypeSet) -> TypeSet {
        let cards = CardTable::new([&HOLDER]);
        let state = GameState::new(2, 0);
        let view = View { cards: &cards, state: &state, log: &[], stop: Layer::Type };
        let mut chars = *printed(&cards, cards.find("Holder").unwrap(), false);
        chars.types = start;
        by_dependency(&view, ObjectId(1), pending.to_vec(), &mut chars);
        chars.types
    }

    fn types(of: &[CardType]) -> TypeSet {
        TypeSet::of(of)
    }

    /// CR 613.8a: "creatures are artifacts" depends on a later effect that makes the object a creature, and waits for it.
    #[test]
    fn a_static_ability_waits_for_the_later_effect_that_changes_what_it_applies_to() {
        let source = ObjectId(2);
        let pending = [Entry::Static(source, &CREATURES_ARE_ARTIFACTS), Entry::Effect(effect(9, EffectKind::AddTypes(types(&[CardType::Creature]))))];
        assert_eq!(types_after(&pending, TypeSet::NONE), types(&[CardType::Creature, CardType::Artifact]));
    }

    /// CR 613.7, 613.9: an effect that changes nothing about what the other applies to leaves them in timestamp order, and
    /// the later type-setting effect replaces what the earlier one added (Oko's Elk on a creature).
    #[test]
    fn independent_effects_keep_their_timestamp_order() {
        let pending = [
            Entry::Static(ObjectId(2), &CREATURES_ARE_ARTIFACTS),
            Entry::Effect(effect(9, EffectKind::SetTypes(types(&[CardType::Creature])))),
        ];
        assert_eq!(types_after(&pending, types(&[CardType::Creature])), types(&[CardType::Creature]));
    }

    /// CR 613.8b: two effects that each change what the other applies to form a loop, and apply in timestamp order: the
    /// earlier applies, and then the later no longer applies to the object.
    #[test]
    fn a_dependency_loop_applies_in_timestamp_order() {
        let one = Entry::Static(ObjectId(2), &NONARTIFACTS_ARE_ENCHANTMENTS);
        let other = Entry::Static(ObjectId(3), &NONENCHANTMENTS_ARE_ARTIFACTS);
        assert_eq!(types_after(&[one, other], TypeSet::NONE), types(&[CardType::Enchantment]));
        assert_eq!(types_after(&[other, one], TypeSet::NONE), types(&[CardType::Artifact]));
    }

    /// CR 613.8b, 613.8c: a chain of dependencies, latest first: "creatures are artifacts" waits for "lands are creatures",
    /// which waits for the effect that makes the object a land; after each, the order is worked out again.
    #[test]
    fn a_chain_of_dependencies_applies_from_its_end() {
        let pending = [
            Entry::Static(ObjectId(2), &CREATURES_ARE_ARTIFACTS),
            Entry::Static(ObjectId(3), &LANDS_ARE_CREATURES),
            Entry::Effect(effect(9, EffectKind::AddTypes(types(&[CardType::Land])))),
        ];
        assert_eq!(types_after(&pending, TypeSet::NONE), types(&[CardType::Land, CardType::Creature, CardType::Artifact]));
    }

    /// CR 613.8b: an effect in a loop that also depends on one outside it waits for that one; the loop's other member, which
    /// depends only on it, goes first though it is the later.
    #[test]
    fn a_loop_member_still_waits_for_what_it_depends_on_outside_the_loop() {
        // "Nonartifacts are enchantments" and "nonenchantments are artifacts" depend on each other; the first also depends on
        // the latest, an effect that makes the object an artifact. So "nonenchantments are artifacts" applies first, and
        // then the first no longer applies.
        let pending = [
            Entry::Static(ObjectId(2), &NONARTIFACTS_ARE_ENCHANTMENTS),
            Entry::Static(ObjectId(3), &NONENCHANTMENTS_ARE_ARTIFACTS),
            Entry::Effect(effect(9, EffectKind::AddTypes(types(&[CardType::Artifact])))),
        ];
        assert_eq!(types_after(&pending, TypeSet::NONE), types(&[CardType::Artifact]));
    }
}
