//! An optional cost to cast a spell that may be paid any number of times (CR 702.56a: replicate another object gives it,
//! `OptionalCost::repeated`), over test cards. Each instance of it is paid separately and triggers on the payments made for
//! it (CR 702.56b): two instances paid twice and once trigger with two and one, whichever is announced first. The number
//! of payments has no bound play can reach: forty {C} pay a {1} forty times.

use crate::{Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};
use std::collections::VecDeque;

const YOU: PlayerId = PlayerId(0);

static FILLER: CardDef = CardDef { name: "Repeated Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

/// The first instance's payments: a life each.
fn gain_one_each(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, item.data[0] as _);
    Ok(())
}

/// The second instance's payments: ten life each.
fn gain_ten_each(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, (10 * item.data[0]) as _);
    Ok(())
}

/// A sorcery for {0} with two abilities that each offer an additional {1} any number of times, each triggering as it is
/// cast with its own count (the shape of a spell given replicate by Hugh, First of Many and by a copy of it).
static SPELL: CardDef = CardDef {
    name: "Repeated Test Spell",
    types: TypeSet::of(&[CardType::Sorcery]),
    has_cost: true,
    abilities: &[
        paid_as_it_is_cast(OptionalCost::repeated(mana("{1}")), gain_one_each, "Pay {1} any number of times: you gain 1 life for each."),
        paid_as_it_is_cast(OptionalCost::repeated(mana("{1}")), gain_ten_each, "Pay {1} any number of times: you gain 10 life for each."),
    ],
    ..CardDef::BLANK
};

/// Your precombat main phase, `colorless` {C} in your pool and the Spell in your hand.
fn game(colorless: u32) -> (Game, ObjectId) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &SPELL])));
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
    let spell = g.state.push_obj(Obj::new(table.id_of(&SPELL), YOU, Zone::hand(YOU)));
    g.state.zone_push(Zone::hand(YOU), spell);
    g.state.add_mana(YOU, Mana::C, colorless, ObjectId::NONE, None, ManaLasts::Step, false);
    (g, spell)
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

/// Casts the Spell announcing `counts` for its instances in turn, pays from the pool and lets everything resolve: the most
/// each announcement allowed.
fn cast(g: &mut Game, spell: ObjectId, counts: &[u32]) -> Vec<u32> {
    let mut counts: VecDeque<u32> = counts.iter().copied().collect();
    let mut mosts = Vec::new();
    let r = next(g);
    let RequestKind::Priority { options } = &r.kind else { panic!("priority first, got {:?}", r.kind) };
    let o = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(spell)).expect("the cast is offered");
    g.answer(r.id, r.player, Answer::Act(Action::of(o))).expect("the cast");
    loop {
        let r = next(g);
        match &r.kind {
            RequestKind::PayCosts { announcing: true, most, .. } => {
                mosts.push(*most);
                let n = counts.pop_front().expect("a count for each instance");
                g.answer(r.id, r.player, Answer::Announce(n)).expect("a count it allows");
            }
            RequestKind::ChooseCostPayment { choices, .. } => {
                let chosen = choices.iter().position(|c| c.kind == CostPaymentKind::Mana).unwrap_or(0);
                g.answer(r.id, r.player, Answer::CostPayment(chosen)).expect("a legal cost order");
            }
            RequestKind::PayCosts { options, .. } => {
                let o = options.iter().find(|o| o.kind == ActionKind::AssignMana)
                    .or_else(|| options.iter().find(|o| o.kind == ActionKind::ConfirmCasting))
                    .expect("the payment goes on");
                g.answer(r.id, r.player, Answer::Act(Action::of(o))).expect("a legal payment");
            }
            RequestKind::OrderTriggers { sources, .. } => {
                g.answer(r.id, r.player, Answer::Order((0..sources.len() as u32).collect())).expect("an order");
            }
            RequestKind::Priority { .. } if g.state.stack.is_empty() => return mosts,
            RequestKind::Priority { .. } => g.answer(r.id, r.player, Answer::Pass).unwrap(),
            other => panic!("unexpected {other:?}"),
        }
    }
}

/// CR 702.56b: each instance triggers on its own payments - twice and once is 2 + 10 life, once and twice 1 + 20, never a
/// count one instance's payments wrote over the other's.
#[test]
fn each_instance_of_a_repeated_cost_triggers_on_its_own_payments() {
    for (counts, gained) in [([2, 1], 12), ([1, 2], 21), ([0, 3], 30), ([3, 0], 3)] {
        let (mut g, spell) = game(6);
        let life = g.state.player(YOU).life;
        cast(&mut g, spell, &counts);
        assert_eq!(g.state.player(YOU).life, life + gained, "paid {counts:?}");
        assert_eq!(g.state.player(YOU).pool.get(Mana::C), 6 - counts.iter().sum::<u32>(), "each payment paid {{1}}");
    }
}

/// CR 702.56a: "any number of times" - forty {C} pay the {1} forty times; the announcement is bounded by nothing play can
/// reach, and the payment judges what the pool can pay.
#[test]
fn a_repeated_cost_may_be_paid_as_many_times_as_the_caster_can_pay() {
    let (mut g, spell) = game(40);
    let life = g.state.player(YOU).life;
    let mosts = cast(&mut g, spell, &[40, 0]);
    assert!(mosts.iter().all(|&most| most >= 1_000_000), "no practical bound on the count: {mosts:?}");
    assert_eq!(g.state.player(YOU).life, life + 40);
    assert_eq!(g.state.player(YOU).pool.get(Mana::C), 0);
}
