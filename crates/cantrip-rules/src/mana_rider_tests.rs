//! Mana a "when that mana is spent" delayed ability rides (CR 603.7a, 106.3), over test cards: each rider is a delayed
//! ability of its own, created with its own words at its own moment (CR 603.7), so mana of two riders - the same source's
//! included - is never interchangeable, and which of them pays is the payer's choice (CR 601.2h). Pyromancer's Goggles
//! tapped, its "red" rewritten to "blue" (Sleight of Mind), untapped and tapped again holds two riders of one source that
//! do different things; here one source's two abilities stand in for them: one rider gains a life, the other loses one.
//! The choice is offered, each answer spends the rider it names, the game-log tree undoes it, and a take-back (CR 733.1)
//! replays a kept mana ability's payment with the very rider it spent.

use crate::{Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);

static FILLER: CardDef = CardDef { name: "Rider Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn on_a_spell_cast(_: &dyn Query, _: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { .. } => Fired::PLAIN,
        _ => None,
    }
}

fn gain_one(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

fn lose_one(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 1);
    Ok(())
}

/// Two mana abilities, each adding {R} that its own "when that mana is spent" ability rides. Their cost is life, not {T},
/// so that one source makes riders again and again (Goggles untapped by Voltaic Key).
static PROBE: CardDef = CardDef {
    name: "Rider Test Probe",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[
        mana_ability(life(1), add(Mana::R).when_spent(0), "Pay 1 life: Add {R}. When that mana is spent to cast a spell, you gain 1 life."),
        mana_ability(life(1), add(Mana::R).when_spent(1), "Pay 1 life: Add {R}. When that mana is spent to cast a spell, you lose 1 life."),
    ],
    delayed: &[
        triggered(when_asked(EventMask::SPELL_CAST, on_a_spell_cast), gain_one, "When that mana is spent to cast a spell, you gain 1 life."),
        triggered(when_asked(EventMask::SPELL_CAST, on_a_spell_cast), lose_one, "When that mana is spent to cast a spell, you lose 1 life."),
    ],
    ..CardDef::BLANK
};

fn red_and_draw(g: &mut dyn cantrip_api::Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::R, 1);
    g.draw(asker.controller, 1);
    Ok(())
}

/// "{1}, {T}, Sacrifice this artifact: Add {R}. Draw a card." (Chromatic Sphere's shape): a draw no take-back reverses.
static SPHERE: CardDef = CardDef {
    name: "Rider Test Sphere",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[mana_ability(cost("{1}, {T}").and(SACRIFICE_THIS), add(Mana::R).by(red_and_draw), "{1}, {T}, Sacrifice this artifact: Add {R}. Draw a card.")],
    ..CardDef::BLANK
};

/// A 1/1 for {R}.
static IMP: CardDef = CardDef {
    name: "Rider Test Imp",
    cost: mana("{R}"),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// A 3/3 for {R}{R}{R}: more than the riders floating, so the mana ability window stays open.
static BRUTE: CardDef = CardDef {
    name: "Rider Test Brute",
    cost: mana("{R}{R}{R}"),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    ..CardDef::BLANK
};

fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &PROBE, &SPHERE, &IMP, &BRUTE])));
    let deck = || vec![table.id_of(&FILLER); 20];
    let mut g = Game::new(table, &Setup { decks: vec![deck(), deck()], ..Default::default() });
    g.verify = true;
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    g.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    g
}

fn put(g: &mut Game, def: &'static CardDef, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), YOU, zone));
    g.state.zone_push(zone, id);
    id
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

fn options(r: &Request) -> Vec<ActionOption> {
    match &r.kind {
        RequestKind::Priority { options } | RequestKind::PayCosts { options, .. } => options.clone(),
        other => panic!("expected priority or a payment, got {other:?}"),
    }
}

/// Answers with the first option `pick` accepts, closing a mana ability window and taking a cost order on the way.
fn choose(g: &mut Game, pick: impl Fn(&ActionOption) -> bool, what: &str) {
    loop {
        let r = next(g);
        if let RequestKind::ChooseCostPayment { choices, .. } = &r.kind {
            let chosen = choices.iter().position(|c| c.kind == CostPaymentKind::Tap)
                .or_else(|| choices.iter().position(|c| c.kind == CostPaymentKind::Mana))
                .or_else(|| choices.iter().position(|c| c.kind != CostPaymentKind::Sacrifice)).unwrap_or(0);
            g.answer(r.id, r.player, Answer::CostPayment(chosen)).expect("a legal cost order");
            continue;
        }
        let offered = options(&r);
        if let Some(o) = offered.iter().find(|o| pick(o)) {
            g.answer(r.id, r.player, Answer::Act(Action::of(o))).expect("a legal answer");
            return;
        }
        if matches!(r.kind, RequestKind::PayCosts { mana_abilities: true, .. })
            && let Some(confirm) = offered.iter().find(|o| o.kind == ActionKind::ConfirmCasting)
        {
            g.answer(r.id, r.player, Answer::Act(Action::of(confirm))).expect("close the mana window");
            continue;
        }
        panic!("{what} not offered: {:?}", offered.iter().map(|o| (o.kind, o.source, o.index)).collect::<Vec<_>>());
    }
}

/// Activates `source`'s `index`th mana ability, at priority or while paying.
fn activate(g: &mut Game, source: ObjectId, index: u8) {
    choose(g, |o| o.kind == ActionKind::ActivateAbility && o.source == Some(source) && o.index == index, "the mana ability");
}

/// The payment question that asks which pool mana pays, past any mana ability window.
fn assignment(g: &mut Game) -> Request {
    loop {
        let r = next(g);
        match &r.kind {
            RequestKind::PayCosts { mana_abilities: true, options, .. } => {
                let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting).expect("the window closes");
                g.answer(r.id, r.player, Answer::Act(Action::of(confirm))).expect("close the mana window");
            }
            RequestKind::ChooseCostPayment { choices, .. } => {
                let chosen = choices.iter().position(|c| c.kind == CostPaymentKind::Mana).unwrap_or(0);
                g.answer(r.id, r.player, Answer::CostPayment(chosen)).expect("a legal cost order");
            }
            RequestKind::PayCosts { .. } => return r,
            other => panic!("expected a payment, got {other:?}"),
        }
    }
}

/// Passes priority until the stack is empty.
fn resolve(g: &mut Game) {
    loop {
        let r = next(g);
        if g.state.stack.is_empty() {
            return;
        }
        assert!(matches!(r.kind, RequestKind::Priority { .. }), "unexpected {:?}", r.kind);
        g.answer(r.id, r.player, Answer::Pass).unwrap();
    }
}

/// The rider of each row of ordinary mana in the pool, with its {R}.
fn riders(g: &Game) -> Vec<(ObjectId, u32, u32)> {
    let mut rows: Vec<(ObjectId, u32, u32)> = g.state.mana_from_of(YOU).map(|e| (e.source, e.rider, e.pool.get(Mana::R))).collect();
    rows.sort();
    rows
}

/// The Probe activated for its first ability, then for its second: {R}{R}, of two riders. The two riders' numbers, the
/// first ability's first.
fn two_riders(g: &mut Game, probe: ObjectId) -> (u32, u32) {
    activate(g, probe, 0);
    next(g);
    let first = g.state.mana_from_of(YOU).find(|e| e.source == probe).map(|e| e.rider).expect("its mana rides");
    activate(g, probe, 1);
    next(g);
    let second = g.state.mana_from_of(YOU).map(|e| e.rider).find(|&r| r != first).expect("a second rider");
    assert!(first != 0 && second != 0, "both ride");
    assert_eq!(riders(g), {
        let mut v = vec![(probe, first, 1), (probe, second, 1)];
        v.sort();
        v
    }, "one source's two riders are kept apart in the pool");
    (first, second)
}

/// CR 601.2h: paying {R} with {R}{R} floating from one source, each riding its own delayed ability, the payer chooses
/// which pays; each answer spends that rider, whose ability alone triggers. The game-log tree undoes the payment.
#[test]
fn two_riders_of_one_source_are_a_choice_and_each_pays_as_chosen() {
    for gains in [true, false] {
        let mut g = game();
        let probe = put(&mut g, &PROBE, Zone::BATTLEFIELD);
        let imp = put(&mut g, &IMP, Zone::hand(YOU));
        let (gain, lose) = two_riders(&mut g, probe);
        let life = g.state.player(YOU).life;
        choose(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(imp), "the cast");
        let r = assignment(&mut g);
        let assign: Vec<ActionOption> = options(&r).into_iter().filter(|o| o.kind == ActionKind::AssignMana).collect();
        let mut offered: Vec<(Option<ObjectId>, u32)> = assign.iter().map(|o| (o.source, o.rider)).collect();
        offered.sort();
        let mut both = vec![(Some(probe), gain), (Some(probe), lose)];
        both.sort();
        assert_eq!(offered, both, "each rider's {{R}} is offered on its own, named by its rider");
        // An answer naming a rider the pool does not hold is refused.
        let mut forged = Action::of(&assign[0]);
        forged.rider = gain.max(lose) + 1;
        assert!(g.answer(r.id, r.player, Answer::Act(forged)).is_err(), "no other rider pays");
        let node = g.at;
        let (spent, kept) = if gains { (gain, lose) } else { (lose, gain) };
        let chosen = assign.iter().find(|o| o.rider == spent).expect("the chosen rider is offered");
        g.answer(r.id, r.player, Answer::Act(Action::of(chosen))).expect("a legal answer");
        resolve(&mut g);
        assert_eq!(riders(&g), vec![(probe, kept, 1)], "the rider not chosen is left in the pool");
        assert!(
            g.log.iter().any(|rec| matches!(rec.event, Event::ManaSpent { source, cast: true, rider, .. } if source == probe && rider == spent)),
            "the history says which rider was spent"
        );
        assert!(!g.log.iter().any(|rec| matches!(rec.event, Event::ManaSpent { rider, .. } if rider == kept)), "and not the other");
        assert_eq!(g.state.player(YOU).life, if gains { life + 1 } else { life - 1 }, "only the spent rider's ability triggered");
        assert!(g.state.battlefield().iter().any(|&id| g.state.obj(id).card == g.cards.id_of(&IMP)), "the Imp resolved");
        let done = g.at;
        // The tree undoes the payment: both riders float again, and redoes it.
        g.goto(node);
        assert_eq!(riders(&g).len(), 2, "undone, both riders are in the pool");
        assert_eq!(g.state.player(YOU).life, life);
        g.goto(done);
        assert_eq!(riders(&g), vec![(probe, kept, 1)]);
        assert_eq!(g.state.player(YOU).life, if gains { life + 1 } else { life - 1 });
    }
}

/// CR 733.1: a cast given up with a mana ability that drew kept, whose {1} the payer paid with one of two riders, is taken
/// back, and the kept ability, activated again, pays with that same rider: the other one stays in the pool.
#[test]
fn a_take_back_pays_a_kept_mana_ability_with_the_rider_it_chose() {
    for first in [true, false] {
        let mut g = game();
        let probe = put(&mut g, &PROBE, Zone::BATTLEFIELD);
        let sphere = put(&mut g, &SPHERE, Zone::BATTLEFIELD);
        let brute = put(&mut g, &BRUTE, Zone::hand(YOU));
        let (gain, lose) = two_riders(&mut g, probe);
        let (spent, kept) = if first { (gain, lose) } else { (lose, gain) };
        let library = g.state.zone(Zone::library(YOU)).len();
        choose(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(brute), "the cast");
        activate(&mut g, sphere, 0);
        // The Sphere's {1}: one of the two riders, asked.
        let r = assignment(&mut g);
        let assign: Vec<ActionOption> = options(&r).into_iter().filter(|o| o.kind == ActionKind::AssignMana).collect();
        assert_eq!(assign.len(), 2, "both riders are offered for the Sphere's {{1}}");
        let chosen = assign.iter().find(|o| o.rider == spent).expect("the chosen rider is offered");
        g.answer(r.id, r.player, Answer::Act(Action::of(chosen))).expect("a legal answer");
        assert_eq!(riders(&g).iter().filter(|&&(s, ..)| s == probe).map(|&(_, r, _)| r).collect::<Vec<_>>(), vec![kept]);
        choose(&mut g, |o| o.kind == ActionKind::CancelCasting, "taking the cast back");
        assert!(matches!(next(&mut g).kind, RequestKind::Priority { .. }), "priority again once the cast is taken back");
        assert_eq!(g.state.zone(Zone::library(YOU)).len(), library - 1, "the Sphere's draw stays");
        assert!(g.state.zone(Zone::hand(YOU)).contains(&brute), "the Brute is back in the hand");
        let mut want = vec![(probe, kept, 1), (sphere, 0, 1)];
        want.sort();
        assert_eq!(riders(&g), want, "replayed, the Sphere spent the rider chosen; the other floats beside the Sphere's {{R}}");
    }
}

/// CR 733.1: a rider made during a cast that is taken back, by a mana ability that is kept, is made again under its number
/// as the kept ability is activated again, though a reversed one made a rider before it: the kept payment it paid names it.
#[test]
fn a_rider_made_by_a_kept_mana_ability_keeps_its_number_through_a_take_back() {
    let mut g = game();
    let reversed = put(&mut g, &PROBE, Zone::BATTLEFIELD);
    let kept = put(&mut g, &PROBE, Zone::BATTLEFIELD);
    // A third, never activated, so that the Brute is offered as affordable.
    put(&mut g, &PROBE, Zone::BATTLEFIELD);
    let sphere = put(&mut g, &SPHERE, Zone::BATTLEFIELD);
    let brute = put(&mut g, &BRUTE, Zone::hand(YOU));
    let life = g.state.player(YOU).life;
    choose(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(brute), "the cast");
    activate(&mut g, reversed, 0);
    activate(&mut g, kept, 0);
    activate(&mut g, sphere, 0);
    let r = assignment(&mut g);
    let rider = g.state.mana_from_of(YOU).find(|e| e.source == kept).map(|e| e.rider).expect("its mana rides");
    let chosen = options(&r).into_iter().find(|o| o.kind == ActionKind::AssignMana && o.source == Some(kept)).expect("its rider is offered");
    assert_eq!(chosen.rider, rider);
    g.answer(r.id, r.player, Answer::Act(Action::of(&chosen))).expect("a legal answer");
    choose(&mut g, |o| o.kind == ActionKind::CancelCasting, "taking the cast back");
    assert!(matches!(next(&mut g).kind, RequestKind::Priority { .. }), "priority again once the cast is taken back");
    assert_eq!(g.state.player(YOU).life, life - 1, "the Probe whose mana paid for nothing kept is reversed, its life back; the other is kept");
    assert!(!g.state.mana_from_of(YOU).any(|e| e.source == reversed), "the reversed Probe's mana is gone");
    assert_eq!(riders(&g), vec![(sphere, 0, 1)], "the kept Probe's rider paid the Sphere again; the reversed one's is gone");
    assert!(
        g.state.delayed.iter().any(|d| d.when == DelayedWhen::ThatMana(rider) && d.source == kept),
        "the kept Probe's ability was made again under its number"
    );
}
