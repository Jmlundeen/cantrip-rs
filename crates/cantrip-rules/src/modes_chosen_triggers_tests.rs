//! Choosing modes emits one ModesChosen event for triggered abilities (CR 700.2, 603.2).
use crate::actions::priority_options;
use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_mechanics::{statics, station, targets};
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);

static FILLER: CardDef = CardDef {
    name: "Dilemma Trigger Filler",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Enter Trigger Opponent Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

fn noop(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

fn gain_one(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

static MODES: ModalDef = modes(&[mode(noop, "First mode."), mode(noop, "Second mode.")]);

static OPTIONAL_MODES: ModalDef = MODES.choose(0, 1);

static WATCHER: CardDef = CardDef {
    name: "Dilemma Trigger Watcher",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[triggered(
        triggers::YOU_FACE_A_DILEMMA,
        gain_one,
        "Whenever you face a dilemma, you gain 1 life.",
    )],
    ..CardDef::BLANK
};

static MODAL_SPELL: CardDef = CardDef {
    name: "Dilemma Trigger Spell",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODES),
    ..CardDef::BLANK
};

static OPTIONAL_MODAL_SPELL: CardDef = CardDef {
    name: "Optional Dilemma Trigger Spell",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    modal: Some(&OPTIONAL_MODES),
    ..CardDef::BLANK
};

static MODAL_ETB: CardDef = CardDef {
    name: "Dilemma Trigger Creature",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODES,
        "When this creature enters, choose one — First mode; or Second mode.",
    )],
    ..CardDef::BLANK
};

fn tap_and_stun(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(target) = targets::object(g, item, 0) {
        g.tap(target);
        g.add_counters(target, CounterKind::Stun, 1);
    }
    Ok(())
}

fn flying_and_vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars
        .keywords
        .with(Keyword::Flying)
        .with(Keyword::Vigilance);
}

static TARGET_AND_STATION: CardDef = CardDef {
    name: "Enter Trigger Station Test",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[
        triggered(THIS_ENTERS, tap_and_stun, "When this artifact enters, tap target creature an opponent controls and put a stun counter on it.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this artifact.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<10>, station::is_a_creature, "10+ | Flying, vigilance (5/6)"),
        static_ability(Layer::Ability, station::charged::<10>, flying_and_vigilance, "10+ | Flying, vigilance (5/6)").continues(),
        static_ability(Layer::PtSet, station::charged::<10>, statics::base_pt::<5, 6>, "10+ | Flying, vigilance (5/6)").continues(),
    ],
    ..CardDef::BLANK
};

fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([
        &FILLER,
        &BEAR,
        &WATCHER,
        &MODAL_SPELL,
        &OPTIONAL_MODAL_SPELL,
        &MODAL_ETB,
        &TARGET_AND_STATION,
    ])));
    let deck = || vec![table.id_of(&FILLER); 20];
    let mut g = Game::new(
        table,
        &Setup {
            decks: vec![deck(), deck()],
            ..Default::default()
        },
    );
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    g.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    g.state.update_player(YOU, |p| p.turn_began = 1);
    g
}

fn put(g: &mut Game, def: &'static CardDef, owner: PlayerId, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), owner, zone));
    g.state.zone_push(zone, id);
    id
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

fn answer(g: &mut Game, r: &Request, a: Answer) {
    g.answer(r.id, r.player, a).expect("a legal answer");
}

fn cast(g: &mut Game, source: ObjectId) {
    let option = {
        let mut cx = Cx {
            g,
            frame: usize::MAX,
            cursor: 0,
            mana_source: None,
            entering: None,
            lineage: Vec::new(),
            asked_at: None,
            doer: None,
            producing: None,
        };
        priority_options(&mut cx, YOU)
            .into_iter()
            .find(|o| o.kind == ActionKind::CastSpell && o.source == Some(source))
            .expect("spell offered")
    };
    let r = next(g);
    answer(g, &r, Answer::Act(Action::of(&option)));
}

fn choose_modes(g: &mut Game, modes: Vec<u8>) -> Request {
    let r = next(g);
    assert!(
        matches!(r.kind, RequestKind::ChooseModes { .. }),
        "expected mode choice, got {:?}",
        r.kind
    );
    answer(g, &r, Answer::Modes(modes));
    next(g)
}

fn choose_mode(g: &mut Game, mode: u8) -> Request {
    choose_modes(g, vec![mode])
}

#[test]
fn the_dilemma_matcher_collects_an_explicit_modes_chosen_event() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU, Zone::BATTLEFIELD);
    assert_eq!(g.state.battlefield(), &[watcher]);
    let event = Event::ModesChosen {
        player: YOU,
        source: watcher,
        ability: AbilityRef::default(),
        modes: 1,
    };
    assert_eq!(EventMask::of(&event), EventMask::MODES_CHOSEN);
    let mut cx = Cx {
        g: &mut g,
        frame: usize::MAX,
        cursor: 0,
        mana_source: None,
        entering: None,
        lineage: Vec::new(),
        asked_at: None,
        doer: None,
        producing: None,
    };
    let asker = Asker {
        source: watcher,
        controller: YOU,
        by: None,
        spell: false,
    };
    let AbilityDef::Triggered(def) = cx.cards().ability(AbilityRef {
        card: cx.cards().id_of(&WATCHER),
        index: 0,
    }) else {
        panic!("watcher is not triggered")
    };
    assert!(
        def.fires(&cx, &asker, &event).is_some(),
        "the matcher should fire directly"
    );
    cx.emit(event);
    assert_eq!(
        cx.g.state.triggers.len(),
        1,
        "the watcher should match ModesChosen"
    );
}

#[test]
fn casting_a_modal_spell_dispatches_one_modes_chosen_event_to_triggers() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU, Zone::BATTLEFIELD);
    let watcher_ref = AbilityRef {
        card: g.cards.id_of(&WATCHER),
        index: 0,
    };
    let AbilityDef::Triggered(def) = g.cards.ability(watcher_ref) else {
        panic!("watcher is not triggered")
    };
    assert!(def.listens.intersects(EventMask::MODES_CHOSEN));
    assert!(def.functions.contains(ZoneKind::Battlefield));
    let spell = put(&mut g, &MODAL_SPELL, YOU, Zone::hand(YOU));
    cast(&mut g, spell);
    let mut r = choose_mode(&mut g, 0);
    if let RequestKind::PayCosts { options, .. } = &r.kind {
        let confirm = options
            .iter()
            .find(|o| o.kind == ActionKind::ConfirmCasting)
            .expect("confirm casting");
        answer(&mut g, &r, Answer::Act(Action::of(confirm)));
        r = next(&mut g);
    }
    assert!(
        matches!(r.kind, RequestKind::Priority { .. }),
        "expected priority after casting, got {:?}",
        r.kind
    );
    assert_eq!(
        g.log
            .iter()
            .filter(|r| matches!(r.event, Event::ModesChosen { player: YOU, .. }))
            .count(),
        1,
        "exactly one mode set was recorded: {:?}",
        g.log.iter().map(|r| r.event).collect::<Vec<_>>()
    );
    assert_eq!(
        g.state.stack.len(),
        2,
        "the chosen mode spell and dilemma trigger are both on the stack; waiting={:?}",
        g.state.triggers
    );
    assert_eq!(g.state.obj(watcher).controller, YOU);
}

#[test]
fn choosing_no_mode_records_the_announcement_without_triggering_a_dilemma() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU, Zone::BATTLEFIELD);
    let spell = put(&mut g, &OPTIONAL_MODAL_SPELL, YOU, Zone::hand(YOU));
    cast(&mut g, spell);
    let mut r = choose_modes(&mut g, vec![]);
    if let RequestKind::PayCosts { options, .. } = &r.kind {
        let confirm = options
            .iter()
            .find(|o| o.kind == ActionKind::ConfirmCasting)
            .expect("confirm casting");
        answer(&mut g, &r, Answer::Act(Action::of(confirm)));
        r = next(&mut g);
    }
    assert!(
        matches!(r.kind, RequestKind::Priority { .. }),
        "expected priority after casting, got {:?}",
        r.kind
    );
    assert_eq!(
        g.state.stack.len(),
        1,
        "no dilemma trigger is created when no mode was chosen"
    );
    assert!(g.state.triggers.is_empty());
    assert!(
        g.log.iter().any(|r| matches!(
            r.event,
            Event::ModesChosen {
                player: YOU,
                modes: 0,
                ..
            }
        )),
        "zero modes remain in history"
    );
    assert_eq!(g.state.obj(watcher).controller, YOU);
}

#[test]
fn choosing_a_mode_for_a_triggered_modal_ability_dispatches_modes_chosen() {
    let mut g = game();
    put(&mut g, &WATCHER, YOU, Zone::BATTLEFIELD);
    let source = put(&mut g, &MODAL_ETB, YOU, Zone::hand(YOU));
    cast(&mut g, source);
    for _ in 0..2 {
        let r = next(&mut g);
        assert!(
            matches!(r.kind, RequestKind::Priority { .. }),
            "expected priority after creature cast, got {:?}",
            r.kind
        );
        answer(&mut g, &r, Answer::Pass);
    }
    let r = choose_mode(&mut g, 0);
    assert!(
        matches!(r.kind, RequestKind::Priority { .. }),
        "expected priority after trigger mode, got {:?}",
        r.kind
    );
    assert_eq!(
        g.state.stack.len(),
        2,
        "the modal enter trigger and dilemma trigger are both on the stack"
    );
    assert_eq!(
        g.log
            .iter()
            .filter(|r| matches!(r.event, Event::ModesChosen { player: YOU, .. }))
            .count(),
        1
    );
}

#[test]
fn an_enter_trigger_with_static_opponent_target_collects_after_a_zone_change() {
    let mut g = game();
    let opponent_creature = put(&mut g, &BEAR, PlayerId(1), Zone::BATTLEFIELD);
    let another_opponent_creature = put(&mut g, &BEAR, PlayerId(1), Zone::BATTLEFIELD);
    let source = put(&mut g, &TARGET_AND_STATION, YOU, Zone::hand(YOU));
    cast(&mut g, source);
    for _ in 0..2 {
        let r = next(&mut g);
        assert!(
            matches!(r.kind, RequestKind::Priority { .. }),
            "expected cast priority, got {:?}",
            r.kind
        );
        answer(&mut g, &r, Answer::Pass);
    }
    let target_request = next(&mut g);
    let (entry, entering) = g
        .log
        .iter()
        .find_map(|r| match r.event {
            Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield) => {
                Some((r.event, new))
            }
            _ => None,
        })
        .expect("source entered");
    let AbilityDef::Triggered(def) = g.cards.ability(AbilityRef {
        card: g.cards.id_of(&TARGET_AND_STATION),
        index: 0,
    }) else {
        panic!("enter ability is not triggered")
    };
    assert!(def.listens.intersects(EventMask::of(&entry)));
    assert!(def
        .fires(
            &g,
            &Asker {
                source: entering,
                controller: YOU,
                by: None,
                spell: false
            },
            &entry
        )
        .is_some());
    assert!((targets::CREATURE_AN_OPPONENT_CONTROLS.filter)(
        &g,
        &Asker {
            source: entering,
            controller: YOU,
            by: None,
            spell: false
        },
        Target::Object(opponent_creature)
    ));
    let RequestKind::ChooseTarget { candidates, .. } = &target_request.kind else {
        panic!(
            "expected Enterprise-shaped ETB target request, got {:?}",
            target_request.kind
        );
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&Target::Object(opponent_creature)));
    assert!(candidates.contains(&Target::Object(another_opponent_creature)));
    answer(
        &mut g,
        &target_request,
        Answer::Targets(vec![Target::Object(opponent_creature)]),
    );
    let r = next(&mut g);
    assert!(
        matches!(r.kind, RequestKind::Priority { .. }),
        "expected priority after choosing ETB target, got {:?}",
        r.kind
    );
    assert_eq!(
        g.state.stack.len(),
        1,
        "the ETB trigger remains on the stack"
    );
}
