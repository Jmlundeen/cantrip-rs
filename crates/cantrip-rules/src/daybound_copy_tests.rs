//! Daybound a permanent has as a copy (CR 702.145b) and turning over a face that can't be up (CR 712.10), over test cards:
//! the example under CR 712.13a - a double-faced card whose back face is a sorcery, entering at night as a copy of a
//! daybound creature, enters transformed, so it doesn't enter at all - and a permanent that can't transform as night comes
//! is left as it is, without the day and night procedure claiming it acted. The scenarios show Glasspool Mimic copying a
//! daybound creature, at night and by day.

use crate::actions::stack_item;
use crate::day_night::{self, Bound};
use crate::ops;
use crate::{Cx, Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

/// A daybound creature that is no double-faced card (Bird Admirer copied by a Clone, in the example).
static ADMIRER: CardDef = CardDef {
    name: "Daybound Copy Test Admirer",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(2),
    keywords: Keywords::of(&[Keyword::Daybound]),
    ..CardDef::BLANK
};

/// The Admirer's id in the test table: every test builds the same table, so it is the same id.
static ADMIRER_ID: std::sync::OnceLock<CardId> = std::sync::OnceLock::new();

/// "Enters as a copy of the Admirer" (Mystic Reflection's replacement, in the example).
fn as_the_admirer(g: &mut dyn cantrip_api::Game, entering: &mut Entering) -> Result<(), Suspend> {
    let _ = g;
    entering.copy = Some(Copiable::of(*ADMIRER_ID.get().expect("the table is built")));
    Ok(())
}

/// A double-faced card whose back face is a sorcery (Invasion of Kylem, in the example), entering as a copy of the Admirer.
static SORCERY_BACK: CardDef = CardDef { name: "Daybound Copy Test Ritual", has_cost: false, types: TypeSet::of(&[CardType::Sorcery]), ..CardDef::BLANK };
static INVADER: CardDef = CardDef {
    name: "Daybound Copy Test Invader",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    enters: Some(as_the_admirer),
    back: Some(&SORCERY_BACK),
    ..CardDef::BLANK
};
/// A daybound double-faced card whose back face is a sorcery: it can't transform (CR 712.10).
static WOLF: CardDef = CardDef {
    name: "Daybound Copy Test Wolf",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    keywords: Keywords::of(&[Keyword::Daybound]),
    back: Some(&SORCERY_BACK),
    ..CardDef::BLANK
};

/// An ordinary daybound double-faced card: a creature on each face, nightbound on the back.
static HOWLER: CardDef = CardDef {
    name: "Daybound Copy Test Howler",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Nightbound]),
    ..CardDef::BLANK
};
static VILLAGER: CardDef = CardDef {
    name: "Daybound Copy Test Villager",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    keywords: Keywords::of(&[Keyword::Daybound]),
    back: Some(&HOWLER),
    ..CardDef::BLANK
};

fn game(time: DayNight) -> (Game, &'static CardTable) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&ADMIRER, &INVADER, &WOLF, &VILLAGER])));
    ADMIRER_ID.get_or_init(|| table.id_of(&ADMIRER));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    game.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
        t.day_night = time;
    });
    (game, table)
}

fn put(game: &mut Game, table: &CardTable, def: &'static CardDef, zone: Zone) -> ObjectId {
    let id = game.state.push_obj(Obj::new(table.id_of(def), YOU, zone));
    let timestamp = game.state.timestamp();
    game.state.update_obj(id, |o| o.timestamp = timestamp);
    game.state.zone_push(zone, id);
    id
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

/// The example under CR 712.13a: the Invader resolving at night as a copy of the daybound Admirer would enter with its
/// sorcery back face up, so it is put into its owner's graveyard. By day it enters, a copy of the Admirer, front face up.
#[test]
fn a_double_faced_spell_entering_at_night_as_a_copy_of_a_daybound_creature_with_a_sorcery_back_goes_to_the_graveyard() {
    for (time, enters) in [(DayNight::Night, false), (DayNight::Day, true)] {
        let (mut game, table) = game(time);
        let invader = put(&mut game, table, &INVADER, Zone::STACK);
        game.state.stack_push(stack_item(StackKind::Spell, invader, AbilityRef::spell(table.id_of(&INVADER)), YOU, &[]));
        let serial = game.state.stack.last().expect("the Invader").serial;
        game.state.frame_push(cantrip_state::Frame::new(cantrip_state::FrameKind::Resolve { serial }));
        let frame = game.state.agenda.len() - 1;
        let mut cx = Cx { g: &mut game, frame, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None };
        crate::resolve::resolve(&mut cx).expect("nothing is asked");
        let new = cx.s().obj(invader).became.expect("it left the stack");
        assert_eq!(cx.s().obj(new).zone, if enters { Zone::BATTLEFIELD } else { Zone::graveyard(YOU) }, "{time:?}");
        if enters {
            assert_eq!(cx.chars(new).name, ADMIRER.name);
            assert!(!cx.s().obj(new).transformed);
        }
    }
}

/// Put onto the battlefield from a graveyard at night, the same card stays where it is (CR 304.4, 307.4).
#[test]
fn a_double_faced_card_put_onto_the_battlefield_at_night_as_a_copy_of_a_daybound_creature_with_a_sorcery_back_stays() {
    let (mut game, table) = game(DayNight::Night);
    let invader = put(&mut game, table, &INVADER, Zone::graveyard(YOU));
    let mut cx = cx(&mut game);
    ops::put_onto_battlefield(&mut cx, &[(invader, YOU, false)], ZoneReason::Other).expect("nothing is asked");
    assert!(cx.s().zone(Zone::graveyard(YOU)).contains(&invader));
}

/// At night, a daybound permanent whose back face is a sorcery can't transform (CR 712.10): it stays front face up, and
/// the procedure that turns permanents over does not stop at it - the ordinary daybound Villager beside it still
/// transforms (CR 702.145c). Once nothing more can turn, it reports that nothing happened, so the state-based actions
/// settle.
#[test]
fn a_daybound_permanent_with_a_sorcery_back_stays_front_face_up_at_night() {
    let (mut game, table) = game(DayNight::Night);
    let wolf = put(&mut game, table, &WOLF, Zone::BATTLEFIELD);
    let villager = put(&mut game, table, &VILLAGER, Zone::BATTLEFIELD);
    let mut cx = cx(&mut game);
    let bound = [Bound { id: wolf, daybound: true }, Bound { id: villager, daybound: true }];
    assert!(day_night::apply(&mut cx, &bound).expect("nothing is asked"));
    assert!(!cx.s().obj(wolf).transformed);
    assert!(cx.s().obj(villager).transformed);
    assert!(!day_night::apply(&mut cx, &[Bound { id: wolf, daybound: true }]).expect("nothing is asked"));
    assert!(!cx.s().obj(wolf).transformed);
}

/// A double-faced permanent with its back face up that is a copy of a daybound creature (Glasspool Mimic entered at night
/// as one, CR 702.145b) keeps daybound, but is not front face up: night does not turn it over (CR 702.145c), and day does
/// not either, as it has no nightbound (CR 702.145f).
#[test]
fn a_back_face_up_copy_of_a_daybound_creature_is_not_turned_over() {
    for time in [DayNight::Night, DayNight::Day] {
        let (mut game, table) = game(time);
        let villager = put(&mut game, table, &VILLAGER, Zone::BATTLEFIELD);
        let admirer = Copiable::of(table.id_of(&ADMIRER));
        game.state.update_obj(villager, |o| {
            o.transformed = true;
            o.copy = Some(admirer);
        });
        let mut cx = cx(&mut game);
        assert!(cx.chars(villager).has(Keyword::Daybound));
        assert!(!day_night::apply(&mut cx, &[Bound { id: villager, daybound: true }]).expect("nothing is asked"), "{time:?}");
        assert!(cx.s().obj(villager).transformed, "{time:?}");
    }
}

/// A double-faced permanent front face up that is a copy of a nightbound face (Glasspool Mimic copying a Frenzied
/// Trapbreaker) is not back face up: day does not turn it over (CR 702.145f).
#[test]
fn a_front_face_up_copy_of_a_nightbound_face_is_not_turned_over_by_day() {
    let (mut game, table) = game(DayNight::Day);
    let villager = put(&mut game, table, &VILLAGER, Zone::BATTLEFIELD);
    let howler = Copiable::of(table.id_of(&HOWLER));
    game.state.update_obj(villager, |o| o.copy = Some(howler));
    let mut cx = cx(&mut game);
    assert!(cx.chars(villager).has(Keyword::Nightbound));
    assert!(!day_night::apply(&mut cx, &[Bound { id: villager, daybound: false }]).expect("nothing is asked"));
    assert!(!cx.s().obj(villager).transformed);
}
