//! Abilities that look back in time (CR 603.10: leaves-the-battlefield abilities, CR 603.10a, and the others it lists):
//! whether one triggers, and who controls it (CR 603.3a), is judged on the game as it was just before the event -
//! characteristics, control and history - over test cards.

use crate::{Cx, Game, Setup};
use cantrip_api::*;

/// "As long as no creature has died this turn": read off the history (`Query::history`).
fn no_creature_died_this_turn(q: &dyn LayerQuery) -> bool {
    let turn = q.state().turn.number;
    !q.history().all().iter().any(|r| {
        r.turn == turn
            && matches!(r.event, Event::ZoneChange { from, to, .. } if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard))
    })
}

fn creature_while_none_died(q: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
    c.is(CardType::Creature) && no_creature_died_this_turn(q)
}

fn grant(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let card = view.state().obj(source).card;
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

/// "When this creature dies" (CR 700.4, 603.10a).
fn this_dies(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. } if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => Fired::PLAIN,
        _ => None,
    }
}

fn gain_two(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 2);
    Ok(())
}

/// "As long as no creature has died this turn, creatures have 'When this creature dies, you gain 2 life.'"
static VIGIL: CardDef = CardDef {
    name: "Look-back Vigil",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Ability, creature_while_none_died, grant, "")],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, this_dies), gain_two, "When this creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Look-back Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

/// The first Bear to die this turn had the granted ability just before it died - the history then held no death - so it
/// triggers; the second, dying after it, never had it that turn.
#[test]
fn a_granted_leaves_ability_is_judged_on_the_history_before_the_death_it_looks_back_on() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&VIGIL, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    put(&VIGIL);
    let first = put(&BEAR);
    let second = put(&BEAR);
    let mut cx = cx(&mut game);
    let triggered = |cx: &Cx, id: ObjectId| cx.s().triggers.iter().any(|t| t.source == id && t.controller == PlayerId(0));
    crate::ops::move_to(&mut cx, first, Zone::graveyard(PlayerId(0)));
    assert!(triggered(&cx, first));
    crate::ops::move_to(&mut cx, second, Zone::graveyard(PlayerId(0)));
    assert!(!triggered(&cx, second));
}

/// "Whenever another creature dies" (CR 603.10a).
fn another_dies(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. } if *old != source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => Fired::PLAIN,
        _ => None,
    }
}

fn creature(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
    c.is(CardType::Creature)
}

/// "Creatures have 'Whenever another creature dies, you gain 2 life.'"
static WAKE: CardDef = CardDef {
    name: "Look-back Wake",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Ability, creature, grant, "")],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, another_dies), gain_two, "Whenever another creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};

/// The Wake and a Bear are destroyed together: just before, the creature that stays had the Wake's granted "whenever
/// another creature dies", so it triggers for the Bear, though it no longer has the ability once the Wake is gone.
#[test]
fn a_permanent_that_stays_is_judged_with_the_abilities_it_had_just_before() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&WAKE, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    let wake = put(&WAKE);
    let dying = put(&BEAR);
    let staying = put(&BEAR);
    let mut cx = cx(&mut game);
    let to = Zone::graveyard(PlayerId(0));
    crate::ops::move_together(&mut cx, &[crate::ops::Moving::to(wake, to), crate::ops::Moving::to(dying, to)], ZoneReason::Destroy);
    assert!(cx.s().triggers.iter().any(|t| t.source == staying));
    assert!(!cx.chars(staying).abilities().any(|a| a.is_delayed()));
}

/// "Whenever another creature dies, you gain 2 life."
static ARTIST: CardDef = CardDef {
    name: "Look-back Artist",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(0),
    toughness: Some(1),
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, another_dies), gain_two, "Whenever another creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};

/// "... for as long as [the source] remains on the battlefield", Assimilation Aegis's kind of copy.
fn source_on_battlefield(q: &dyn Query, e: &ContinuousEffect) -> bool {
    q.state().battlefield().contains(&e.source)
}

static AEGIS: CardDef = CardDef {
    name: "Look-back Aegis",
    types: TypeSet::of(&[CardType::Artifact]),
    lasting: &[Lasting { text: "", holds: source_on_battlefield }],
    ..CardDef::BLANK
};

/// A Bear is a copy of the Artist for as long as the Aegis remains; the Aegis and another Bear are destroyed together. The
/// copy ends at once (CR 611.2b), but just before the death the Bear was an Artist, so it triggers (CR 603.10a).
#[test]
fn a_copy_that_ends_with_the_event_is_judged_as_it_was_just_before() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&ARTIST, &AEGIS, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    let aegis = put(&AEGIS);
    let copied = put(&BEAR);
    let dying = put(&BEAR);
    let mut cx = cx(&mut game);
    crate::ops::become_copy(&mut cx, aegis, copied, Copiable::of(table.id_of(&ARTIST)), Duration::AsLongAs(table.id_of(&AEGIS), 0));
    assert_eq!(cx.chars(copied).name, "Look-back Artist");
    let to = Zone::graveyard(PlayerId(0));
    crate::ops::move_together(&mut cx, &[crate::ops::Moving::to(aegis, to), crate::ops::Moving::to(dying, to)], ZoneReason::Destroy);
    assert_eq!(cx.chars(copied).name, "Look-back Bear");
    assert!(cx.s().triggers.iter().any(|t| t.source == copied));
}

/// A Bear that is a copy of a Bear "except it has" the Artist's ability (CR 707.9b), for as long as the Aegis remains: the
/// Aegis and another Bear are destroyed together, the copy ends at once, and just before the death the Bear had that
/// ability from its copy's exception, so it triggers.
#[test]
fn an_ability_a_copy_effect_added_is_looked_back_with() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&ARTIST, &AEGIS, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    let aegis = put(&AEGIS);
    let copied = put(&BEAR);
    let dying = put(&BEAR);
    let mut cx = cx(&mut game);
    let values = Copiable::of(table.id_of(&BEAR)).with_ability(AbilityRef { card: table.id_of(&ARTIST), index: 0 });
    crate::ops::become_copy(&mut cx, aegis, copied, values, Duration::AsLongAs(table.id_of(&AEGIS), 0));
    let to = Zone::graveyard(PlayerId(0));
    crate::ops::move_together(&mut cx, &[crate::ops::Moving::to(aegis, to), crate::ops::Moving::to(dying, to)], ZoneReason::Destroy);
    assert!(cx.chars(copied).abilities().next().is_none());
    assert!(cx.s().triggers.iter().any(|t| t.source == copied));
}

/// "As long as you control an artifact, this creature has 'Whenever another creature dies, you gain 2 life.'" - an ability
/// its own static ability gives it from its `delayed` list.
fn while_you_control_an_artifact(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let s = view.state();
    let controller = s.obj(source).controller;
    if s.battlefield().iter().any(|&id| id != source && s.obj(id).controller == controller && view.chars(id).is(CardType::Artifact)) {
        grant(view, source, chars);
    }
}

static SELF_GRANTER: CardDef = CardDef {
    name: "Look-back Self Granter",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    statics: &[characteristic_defining(Layer::Ability, while_you_control_an_artifact, "")],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, another_dies), gain_two, "Whenever another creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};

static RELIC: CardDef = CardDef { name: "Look-back Relic", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

/// The Relic and a Bear are destroyed together: just before, the Granter had the ability its own static ability gives it
/// while you control an artifact, so it triggers, though it no longer has it.
#[test]
fn an_ability_its_own_static_ability_gave_it_is_looked_back_with() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&SELF_GRANTER, &RELIC, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    let granter = put(&SELF_GRANTER);
    let relic = put(&RELIC);
    let dying = put(&BEAR);
    let mut cx = cx(&mut game);
    assert!(cx.chars(granter).abilities().any(|a| a.is_delayed()));
    let to = Zone::graveyard(PlayerId(0));
    crate::ops::move_together(&mut cx, &[crate::ops::Moving::to(relic, to), crate::ops::Moving::to(dying, to)], ZoneReason::Destroy);
    assert!(!cx.chars(granter).abilities().any(|a| a.is_delayed()));
    assert!(cx.s().triggers.iter().any(|t| t.source == granter));
}

/// "Creatures have 'Whenever another creature dies, you gain 2 life'", a static ability that functions from a graveyard.
static HAUNT: CardDef = CardDef {
    name: "Look-back Haunt",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Ability, creature, grant, "").from(ZoneKind::Graveyard)],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, another_dies), gain_two, "Whenever another creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};

/// The Haunt in the graveyard gives creatures the ability; it is exiled as a Bear dies. Just before, the Bear that stays
/// had the ability, so it triggers.
#[test]
fn an_ability_a_static_ability_in_a_graveyard_gave_is_looked_back_with() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&HAUNT, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let put = |game: &mut Game, def: &'static CardDef, zone: Zone| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), zone));
        game.state.zone_push(zone, id);
        game.state.see_statics();
        id
    };
    let haunt = put(&mut game, &HAUNT, Zone::graveyard(PlayerId(0)));
    let staying = put(&mut game, &BEAR, Zone::BATTLEFIELD);
    let dying = put(&mut game, &BEAR, Zone::BATTLEFIELD);
    let mut cx = cx(&mut game);
    assert!(cx.chars(staying).abilities().any(|a| a.is_delayed()));
    crate::ops::move_together(
        &mut cx,
        &[crate::ops::Moving::to(haunt, Zone::exile(PlayerId(0))), crate::ops::Moving::to(dying, Zone::graveyard(PlayerId(0)))],
        ZoneReason::Other,
    );
    assert!(!cx.chars(staying).abilities().any(|a| a.is_delayed()));
    assert!(cx.s().triggers.iter().any(|t| t.source == staying));
}

/// An emblem (CR 114.4) whose static ability gives creatures "Whenever another creature dies, you gain 2 life".
static EMBLEM: CardDef = CardDef {
    name: "Look-back Emblem",
    statics: &[static_ability(Layer::Ability, creature, grant, "")],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, another_dies), gain_two, "Whenever another creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};

/// The emblem in the command zone gives the Bear that stays its ability, so a Bear dying triggers it.
#[test]
fn an_ability_an_emblem_gave_is_looked_back_with() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&EMBLEM, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let put = |game: &mut Game, def: &'static CardDef, zone: Zone| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), zone));
        game.state.zone_push(zone, id);
        game.state.see_statics();
        id
    };
    put(&mut game, &EMBLEM, Zone::COMMAND);
    let staying = put(&mut game, &BEAR, Zone::BATTLEFIELD);
    let dying = put(&mut game, &BEAR, Zone::BATTLEFIELD);
    let mut cx = cx(&mut game);
    assert!(cx.chars(staying).abilities().any(|a| a.is_delayed()));
    crate::ops::move_to(&mut cx, dying, Zone::graveyard(PlayerId(0)));
    assert!(cx.s().triggers.iter().any(|t| t.source == staying));
}

/// A Room's left door, "Whenever another creature dies, you gain 2 life" (Funeral Room's kind), and its right door, blank.
static WAKE_DOOR: CardDef = CardDef {
    name: "Look-back Wake Door",
    types: TypeSet::of(&[CardType::Enchantment]),
    subtypes: SubtypeSet::of(&[Subtype::Room]),
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, another_dies), gain_two, "Whenever another creature dies, you gain 2 life.")],
    ..CardDef::BLANK
};
static BLANK_DOOR: CardDef = CardDef { name: "Look-back Blank Door", abilities: &[], ..WAKE_DOOR };
static WAKE_ROOM: CardDef =
    CardDef { name: "Look-back Wake Door // Look-back Blank Door", split: Some((&WAKE_DOOR, &BLANK_DOOR)), abilities: &[], ..WAKE_DOOR };

/// A Room with its left door unlocked has that door's "whenever another creature dies" (CR 709.5): a Bear dying triggers it.
/// The ability is the door's, not the Room card's own, and it is looked back with all the same (CR 603.10a).
#[test]
fn an_unlocked_doors_leaves_ability_is_looked_back_with() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&WAKE_DOOR, &BLANK_DOOR, &WAKE_ROOM, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    let room = put(&WAKE_ROOM);
    let dying = put(&BEAR);
    game.state.update_obj(room, |o| o.marks.doors = DOOR_LEFT);
    let mut cx = cx(&mut game);
    assert!(cx.chars(room).abilities().next().is_some());
    crate::ops::move_to(&mut cx, dying, Zone::graveyard(PlayerId(0)));
    assert!(cx.s().triggers.iter().any(|t| t.source == room));
}

/// Not a test of the rules: how long a wrath of a hundred creatures takes while a static ability that gives abilities is
/// out, so every permanent is recorded for the look-back pass (`cargo test ... wrath_of_a_hundred -- --ignored --nocapture`).
#[test]
#[ignore]
fn wrath_of_a_hundred_with_a_granting_static() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&WAKE, &BEAR])));
    let rounds = 200;
    let mut total = std::time::Duration::ZERO;
    for _ in 0..rounds {
        let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
        let mut put = |def: &'static CardDef| {
            let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
            game.state.zone_push(Zone::BATTLEFIELD, id);
            game.state.see_statics();
            id
        };
        put(&WAKE);
        let bears: Vec<ObjectId> = (0..100).map(|_| put(&BEAR)).collect();
        let mut cx = cx(&mut game);
        let moves: Vec<crate::ops::Moving> = bears.iter().map(|&b| crate::ops::Moving::to(b, Zone::graveyard(PlayerId(0)))).collect();
        let started = std::time::Instant::now();
        crate::ops::move_together(&mut cx, &moves, ZoneReason::Destroy);
        total += started.elapsed();
    }
    println!(
        "WRATH {:?} per wrath of 100, {} bytes a recorded permanent",
        total / rounds,
        std::mem::size_of::<(ObjectId, Chars, PlayerId)>()
    );
}

fn equipped(q: &dyn LayerQuery, source: ObjectId, id: ObjectId, _: &Chars) -> bool {
    q.state().obj(source).attached_to == Some(id)
}

/// "Whenever an Equipment becomes unattached from this creature" (CR 603.10c).
fn unattached_from_this(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Unattached { from, .. } if *from == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Equipped creature has 'Whenever an Equipment becomes unattached from this creature, you gain 2 life.'"
static TETHER: CardDef = CardDef {
    name: "Look-back Tether",
    types: TypeSet::of(&[CardType::Artifact]),
    statics: &[static_ability(Layer::Ability, equipped, grant, "")],
    delayed: &[triggered(when(EventMask::UNATTACHED, unattached_from_this), gain_two, "Whenever an Equipment becomes unattached from this creature, you gain 2 life.")],
    ..CardDef::BLANK
};

/// An ability that triggers on becoming unattached looks back (CR 603.10c): the creature had the ability the Equipment gave
/// it just before the Equipment came off, so it triggers, though it has no such ability once it is unattached.
#[test]
fn an_unattachment_is_judged_on_the_game_as_it_was_while_attached() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&TETHER, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    let tether = put(&TETHER);
    let bear = put(&BEAR);
    game.state.update_obj(tether, |o| o.attached_to = Some(bear));
    let mut cx = cx(&mut game);
    assert!(cx.chars(bear).abilities().any(|a| a.is_delayed()));
    crate::ops::unattach(&mut cx, tether);
    assert!(cx.s().triggers.iter().any(|t| t.source == bear));
    assert!(!cx.chars(bear).abilities().any(|a| a.is_delayed()));
}

fn creature_while_a_spell_is_on_the_stack(q: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
    c.is(CardType::Creature) && !q.state().stack.is_empty()
}

/// "Whenever a spell is countered" (CR 603.10e).
fn a_spell_is_countered(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Countered { .. } => Fired::PLAIN,
        _ => None,
    }
}

/// "As long as a spell is on the stack, creatures have 'Whenever a spell is countered, you gain 2 life.'"
static HUSH: CardDef = CardDef {
    name: "Look-back Hush",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Ability, creature_while_a_spell_is_on_the_stack, grant, "")],
    delayed: &[triggered(when(EventMask::COUNTERED, a_spell_is_countered), gain_two, "Whenever a spell is countered, you gain 2 life.")],
    ..CardDef::BLANK
};

/// An ability that triggers on a spell being countered looks back (CR 603.10e): the Bear had the Hush's ability just before
/// the only spell left the stack, so it triggers, though it has no such ability once the stack is empty.
#[test]
fn a_countered_spell_is_judged_on_the_game_as_it_was_while_on_the_stack() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&HUSH, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let mut put = |def: &'static CardDef| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        game.state.see_statics();
        id
    };
    put(&HUSH);
    let bear = put(&BEAR);
    let spell = game.state.push_obj(Obj::new(table.id_of(&BEAR), PlayerId(1), Zone::STACK));
    game.state.zone_push(Zone::STACK, spell);
    game.state.stack_push(crate::actions::stack_item(StackKind::Spell, spell, AbilityRef { card: table.id_of(&BEAR), index: 0 }, PlayerId(1), &[]));
    let mut cx = cx(&mut game);
    assert!(cx.chars(bear).abilities().any(|a| a.is_delayed()));
    crate::resolve::counter(&mut cx, spell);
    assert!(cx.s().stack.is_empty());
    assert!(cx.s().triggers.iter().any(|t| t.source == bear));
    assert!(!cx.chars(bear).abilities().any(|a| a.is_delayed()));
}
