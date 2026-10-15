//! Characteristics (CR 613), computed when asked: the copiable values (layer 1, CR 707), then the continuous
//! effects that apply to the object in timestamp order, then counters (layer 7c).
//!
//! Characteristics are not cached per object: a game has a few dozen objects and a handful of effects. What is shared
//! is what does not depend on the object: a card's own values (`CardTable::printed`) and the list of static-ability
//! sources, found once per position (`GameState::epoch`, which every write and undo moves).

use cantrip_api::*;
use cantrip_state::ContinuousEffect;

pub fn chars(cards: &CardTable, state: &GameState, id: ObjectId) -> Chars {
    chars_before(cards, state, id, Layer::End)
}

/// An object's characteristics as if it were the face `face` of its card (CR 715.4, 709.3, 712.8a): a card in hand
/// judged as the Adventure, half or back face it would be cast as. The face's own printed values replace the card's.
pub fn chars_as_face(cards: &CardTable, state: &GameState, id: ObjectId, face: CardId) -> Chars {
    let mut chars = chars(cards, state, id);
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
    stop: Layer,
}

impl LayerQuery for View<'_> {
    fn state(&self) -> &GameState {
        self.state
    }

    fn cards(&self) -> &CardTable {
        self.cards
    }

    fn chars(&self, id: ObjectId) -> Chars {
        chars_before(self.cards, self.state, id, self.stop)
    }
}

/// More permanents with static abilities than any game has at once; past it, the rest are not looked at.
const MAX_STATIC_SOURCES: usize = 32;

const LAYERS: [Layer; 7] =
    [Layer::Type, Layer::Color, Layer::Ability, Layer::PtDefine, Layer::PtSet, Layer::PtModify, Layer::PtSwitch];

/// The characteristics with every layer before `stop` applied (CR 613.1): the copiable values, then each layer's
/// static abilities and other continuous effects in turn, statics before effects, each in timestamp order.
pub fn chars_before(cards: &CardTable, state: &GameState, id: ObjectId, stop: Layer) -> Chars {
    let obj = state.obj(id);
    let spell = obj.zone.is(ZoneKind::Stack);
    // The copiable values (CR 707.2): a copy effect's, or the face of the card that is up (CR 711.2).
    let card = match &obj.copy {
        Some(c) => c.card,
        None => cards.face(obj.card, obj.transformed),
    };
    let def = cards.get(card);
    let start = || {
        let mut chars = match &obj.copy {
            Some(c) => initial(cards, card, Some(c)),
            None => *printed(cards, card, false),
        };
        chars.spell = spell;
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
    let has_effects = on_battlefield && state.effects.iter().any(|e| e.affected == id);
    let has_counters = on_battlefield && obj.counters.iter().any(|&c| c != 0);
    let has_own = !def.statics.is_empty();
    // The permanents (and graveyard cards) whose static abilities could apply, found once for every layer: most games
    // look at a flag and find none.
    let found = (cards.any_statics && state.statics_seen).then(|| static_sources(cards, state));
    let sources: &[(ObjectId, ZoneKind)] = found.as_ref().map_or(&[], |f| &f.list);
    if !has_effects && !has_counters && !has_own && sources.is_empty() {
        if obj.copy.is_none() && !spell {
            // Nothing acts on it: the card's own values, made once.
            return *printed(cards, card, stop > Layer::Ability);
        }
        let mut chars = start();
        if stop > Layer::Ability && obj.copy.is_none() {
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
        return chars;
    }
    let mut chars = start();
    let colors = obj.copy.and_then(|c| c.colors);
    let pt = obj.copy.and_then(|c| c.pt);
    // The continuous effects on it, oldest first.
    let fx: Vec<ContinuousEffect> =
        if has_effects { state.effects.iter().filter(|e| e.affected == id).copied().collect() } else { Vec::new() };
    for layer in LAYERS {
        if layer >= stop {
            break;
        }
        // CR 305.6: the abilities a basic land type carries follow from the types layer 4 settled, as the object's
        // own from the start of layer 6.
        if layer == Layer::Ability && chars.is(CardType::Land) {
            intrinsic_mana(cards, def, &mut chars);
        }
        if layer == Layer::Ability && chars.is(CardType::Battle) {
            intrinsic_siege(cards, &mut chars);
        }
        // Keyword counters first: they have been on the object since before any effect that took its abilities away, and a
        // "loses hexproof" effect has to reach the hexproof a counter gives (CR 613.1f, 122.1b).
        if has_counters {
            counters(obj, layer, &mut chars);
        }
        // CR 613.7: within a layer, static abilities and other continuous effects apply in timestamp order, interleaved.
        if has_own || !sources.is_empty() {
            statics(cards, state, id, def, (colors.is_some(), pt.is_some()), sources, on_battlefield, layer, &fx, &mut chars);
        } else {
            for e in &fx {
                apply_effect(e, layer, &mut chars);
            }
        }
    }
    chars
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
        let note = |list: &mut Vec<(ObjectId, ZoneKind)>, source: ObjectId, from: ZoneKind| {
            let sobj = state.obj(source);
            let scard = sobj.copy.map(|c| c.card).unwrap_or_else(|| cards.face(sobj.card, sobj.transformed));
            // A permanent that has lost all its abilities has no static abilities (CR 613.1f).
            let lost = !state.effects.is_empty()
                && state.effects.iter().any(|e| e.affected == source && e.kind == EffectKind::LoseAllAbilities);
            if cards.statics[scard.0 as usize] && list.len() < MAX_STATIC_SOURCES && live(state, source) && !lost {
                list.push((source, from));
            }
        };
        for &source in state.battlefield() {
            note(&mut made.list, source, ZoneKind::Battlefield);
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
        // CR 613.7: static abilities apply in the timestamp order of the objects that have them.
        made.list.sort_by_key(|&(source, _)| state.obj(source).timestamp);
        made.epoch = state.epoch;
    }
    state.static_sources.borrow()
}

/// The copiable values of `card` as an object has them before any layer applies: the card's own, or a copy effect's
/// (`copy`), with the abilities the card prints.
fn initial(cards: &CardTable, card: CardId, copy: Option<&Copiable>) -> Chars {
    let def = cards.get(card);
    let (extra_types, extra_subtypes, extra_abilities, extra_keywords, colors, pt, no_cost, copy_name) = match copy {
        Some(c) => (c.extra_types, c.extra_subtypes, c.extra_abilities, c.extra_keywords, c.colors, c.pt, c.no_cost, c.name),
        None => (TypeSet::NONE, SubtypeSet::NONE, [None; 4], Keywords::NONE, None, None, false, None),
    };
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
    Chars {
        card,
        name: copy_name.unwrap_or(def.name),
        types: TypeSet(def.types.0 | extra_types.0),
        supertypes: def.supertypes,
        subtypes: def.subtypes.union(extra_subtypes),
        // CR 707.9b: a copy that says what it is gets that, not what the card it copies is (707.9d).
        colors: colors.unwrap_or_else(|| def.colors()),
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
        must_attack: false,
        abilities,
        no_cost,
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
        }
        Box::new(chars)
    })
}

/// Applies the static abilities of `layer` that affect `id`: its own characteristic-defining ones, anywhere, and
/// those of permanents on the battlefield.
fn statics(
    cards: &CardTable,
    state: &GameState,
    id: ObjectId,
    def: &CardDef,
    overrides: (bool, bool),
    sources: &[(ObjectId, ZoneKind)],
    on_battlefield: bool,
    layer: Layer,
    fx: &[ContinuousEffect],
    chars: &mut Chars,
) {
    let view = View { cards, state, stop: layer };
    // CR 707.9d: a copy that states its colors or its power and toughness does not copy a characteristic-defining
    // ability that would define them.
    let overridden = (layer == Layer::Color && overrides.0) || (layer == Layer::PtDefine && overrides.1);
    for st in def.statics.iter().filter(|s| s.own && s.layer == layer) {
        if !overridden {
            (st.apply)(&view, id, chars);
        }
    }
    // The sources are in timestamp order; a continuous effect older than a source's static ability applies before it.
    let mut next = 0;
    for &(source, from) in sources {
        let sobj = state.obj(source);
        let scard = sobj.copy.map(|c| c.card).unwrap_or_else(|| cards.face(sobj.card, sobj.transformed));
        for st in cards.get(scard).statics.iter().filter(|s| !s.own && s.layer == layer && s.from == from) {
            if (on_battlefield || st.everywhere) && (st.affects)(&view, source, id, chars) {
                while next < fx.len() && fx[next].timestamp < sobj.timestamp {
                    apply_effect(&fx[next], layer, chars);
                    next += 1;
                }
                (st.apply)(&view, source, chars);
            }
        }
    }
    for e in &fx[next..] {
        apply_effect(e, layer, chars);
    }
}

/// A continuous effect a resolved spell or ability created (CR 611.2), applied if it is in `layer`.
fn apply_effect(effect: &ContinuousEffect, layer: Layer, chars: &mut Chars) {
    {
        match (layer, effect.kind) {
            // CR 205.1b: added types are added; a land that becomes a creature is still a land.
            (Layer::Type, EffectKind::AddTypes(types)) => chars.types = TypeSet(chars.types.0 | types.0),
            (Layer::Type, EffectKind::AddSubtype(s)) => chars.subtypes = chars.subtypes.with(s),
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
            (Layer::Ability, EffectKind::LoseAllAbilities) => {
                chars.keywords = Keywords::NONE;
                chars.protection = ColorSet::NONE;
                chars.protection_colorless = false;
                chars.protection_types = TypeSet::NONE;
                chars.protection_everything = false;
                chars.must_attack = false;
                chars.abilities = [None; MAX_ABILITIES];
            }
            (Layer::Ability, EffectKind::Protection(colors)) => chars.protection = ColorSet(chars.protection.0 | colors.0),
            (Layer::Ability, EffectKind::ProtectionFromColorless) => chars.protection_colorless = true,
            (Layer::Ability, EffectKind::ProtectionFromTypes(types)) => chars.protection_types = TypeSet(chars.protection_types.0 | types.0),
            (Layer::Ability, EffectKind::ProtectionFromEverything) => chars.protection_everything = true,
            (Layer::Ability, EffectKind::ProtectionFromPlayer(p)) => chars.protection_players |= 1 << p.0,
            (Layer::Ability, EffectKind::AddKeywords(k)) => chars.keywords = Keywords(chars.keywords.0 | k.0),
            // CR 613.1f: lost at this effect's timestamp, so a keyword granted after it is kept.
            (Layer::Ability, EffectKind::LoseKeywords(k)) => chars.keywords = Keywords(chars.keywords.0 & !k.0),
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

/// Counters' effects: keyword counters in layer 6 (CR 613.1f, 122.1b), +1/+1 and -1/-1 in 7c (CR 613.4c).
fn counters(obj: &Obj, layer: Layer, chars: &mut Chars) {
    match layer {
        Layer::Ability => {
            for kind in CounterKind::KEYWORDS {
                if obj.counters(kind) > 0
                    && let Some(k) = kind.keyword()
                {
                    chars.keywords = chars.keywords.with(k);
                }
            }
        }
        Layer::PtModify => {
            let plus = obj.counters(CounterKind::PlusOne) - obj.counters(CounterKind::MinusOne);
            if plus != 0 {
                chars.power = chars.power.map(|v| v + plus);
                chars.toughness = chars.toughness.map(|v| v + plus);
            }
        }
        _ => {}
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
        .position(|a| !matches!(a, Some(r) if matches!(cards.ability(*r), AbilityDef::Activated(d) if d.mana.is_some())))
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
pub fn can_target(cards: &CardTable, state: &GameState, asker: &Asker, target: Target) -> bool {
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
                let target_chars = chars(cards, state, id);
                // Shroud: no player may target it (CR 702.18a); hexproof: no opponent (CR 702.11b).
                if target_chars.has(Keyword::Shroud) {
                    return false;
                }
                if target_chars.has(Keyword::Hexproof) && obj.controller != asker.controller {
                    return false;
                }
                let mut source_chars = chars(cards, state, asker.source);
                // A card that is not on the battlefield or the stack and asks for a target is about to be cast: it
                // moves to the stack before its targets are chosen (CR 601.2a), so it is judged as the spell it
                // will be, which protection from spells (CR 702.16a) turns away.
                let source = state.obj(asker.source);
                if !source.zone.is(ZoneKind::Battlefield) && !source.zone.is(ZoneKind::Stack) {
                    source_chars.spell = true;
                }
                if target_chars.protected_from_by(&source_chars, asker.controller) {
                    return false;
                }
            }
            true
        }
    }
}

/// Summoning sickness (CR 302.6): a creature that has not been under its controller's control continuously since
/// their most recent turn began, and has no haste.
pub fn summoning_sick(cards: &CardTable, state: &GameState, id: ObjectId) -> bool {
    let obj = state.obj(id);
    let c = chars(cards, state, id);
    c.is(CardType::Creature) && !c.has(Keyword::Haste) && obj.controlled_since >= state.player(obj.controller).turn_began
}
