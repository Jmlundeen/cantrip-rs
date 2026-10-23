//! A cap on how many creatures may attack one player, planeswalker or battle (CR 508.1c), from a static ability
//! (`CardDef::attack_limits`): read off the battlefield for every attack target, with the Asker of the permanent that has
//! it, the smallest where two cap one target, nothing once that permanent loses its abilities, and carried into the
//! declaration of attackers the attacking player is asked for. Over test cards; Tomik, Orzhov Lawmage prints the
//! planeswalker form ("Planeswalkers you control have 'No more than one creature can attack this planeswalker each
//! combat.'"), Judoon Enforcers, Mirri, Weatherlight Duelist and The Eternal Wanderer print others.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::{Game, Setup};
use cantrip_api::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

static FILLER: CardDef = CardDef { name: "Attack Limit Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
static BEAR: CardDef =
    CardDef { name: "Attack Limit Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };
/// Enters with its printed loyalty (CR 306.5b), so it stays on the battlefield to be attacked.
static WALKER: CardDef = CardDef { name: "Attack Limit Walker", types: TypeSet::of(&[CardType::Planeswalker]), loyalty: Some(3), ..CardDef::BLANK };

/// "No more than one creature can attack this planeswalker each combat", of the planeswalkers its controller controls.
fn one_walker(q: &dyn Query, asker: &Asker, target: Target) -> Option<u32> {
    matches!(target, Target::Object(id) if q.obj(id).controller == asker.controller && q.chars(id).is(CardType::Planeswalker)).then_some(1)
}

/// The same cap of two, which one above lowers.
fn two_walkers(q: &dyn Query, asker: &Asker, target: Target) -> Option<u32> {
    one_walker(q, asker, target).map(|_| 2)
}

static CAPPED: CardDef =
    CardDef { name: "Attack Limit Capped", types: TypeSet::of(&[CardType::Enchantment]), attack_limits: &[one_walker], ..CardDef::BLANK };
static CAPPED_TWO: CardDef =
    CardDef { name: "Attack Limit Capped Two", types: TypeSet::of(&[CardType::Enchantment]), attack_limits: &[two_walkers], ..CardDef::BLANK };

/// What the last rule was asked with: the permanent that has the ability, and its controller.
static ASKED_SOURCE: AtomicU32 = AtomicU32::new(u32::MAX);
static ASKED_CONTROLLER: AtomicU32 = AtomicU32::new(u32::MAX);

fn records_asker(_: &dyn Query, asker: &Asker, _: Target) -> Option<u32> {
    ASKED_SOURCE.store(asker.source.0, Ordering::Relaxed);
    ASKED_CONTROLLER.store(u32::from(asker.controller.0), Ordering::Relaxed);
    None
}

static RECORDER: CardDef =
    CardDef { name: "Attack Limit Recorder", types: TypeSet::of(&[CardType::Enchantment]), attack_limits: &[records_asker], ..CardDef::BLANK };

fn table() -> &'static CardTable {
    Box::leak(Box::new(CardTable::new([&FILLER, &BEAR, &WALKER, &CAPPED, &CAPPED_TWO, &RECORDER])))
}

/// Runs the game, passing and declaring nothing, until `stop` says the question asked is the one wanted.
fn until(g: &mut Game, stop: impl Fn(&Game, &Request) -> bool) -> Request {
    for _ in 0..200 {
        g.run();
        let request = g.pending().cloned().expect("a question");
        if stop(g, &request) {
            return request;
        }
        let answer = match &request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            other => panic!("unexpected question {other:?}"),
        };
        g.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the question never came");
}

/// A game of two: `permanents` on P0's battlefield (each `lost`, if it has lost all its abilities), P0's Walker cast in its
/// first main phase, and `bears` attacking for P1 - the declaration of attackers P1 is asked for on its own turn.
fn game(permanents: &[(&'static CardDef, bool)], bears: usize) -> (Game, Request, Vec<ObjectId>) {
    let table = table();
    let filler = table.id_of(&FILLER);
    let mut deck = vec![table.id_of(&WALKER)];
    deck.extend(std::iter::repeat_n(filler, 30));
    let mut g = Game::new(table, &Setup { decks: vec![deck, vec![filler; 30]], ..Default::default() });
    g.verify = true;
    for &(def, lost) in permanents {
        let id = g.state.push_obj(Obj::new(table.id_of(def), P0, Zone::BATTLEFIELD));
        g.state.zone_push(Zone::BATTLEFIELD, id);
        if lost {
            let timestamp = g.state.timestamp();
            g.state.effect_push(ContinuousEffect { source: id, controller: P0, timestamp, affected: id, kind: EffectKind::LoseAllAbilities, duration: Duration::Indefinite });
        }
    }
    let bears = (0..bears)
        .map(|_| {
            let id = g.state.push_obj(Obj::new(table.id_of(&BEAR), P1, Zone::BATTLEFIELD));
            g.state.zone_push(Zone::BATTLEFIELD, id);
            id
        })
        .collect();
    let main = until(&mut g, |g, r| r.player == P0 && g.state.turn.step() == Step::PrecombatMain && matches!(r.kind, RequestKind::Priority { .. }));
    let RequestKind::Priority { options } = &main.kind else { unreachable!() };
    let cast = options
        .iter()
        .find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| g.state.obj(s).card == g.cards.id_of(&WALKER)))
        .expect("the Walker is offered");
    g.answer(main.id, P0, Answer::Act(Action::of(cast))).unwrap();
    until(&mut g, |g, r| g.state.stack.is_empty() && r.player == P0 && matches!(r.kind, RequestKind::Priority { .. }));
    let ask = until(&mut g, |_, r| matches!(r.kind, RequestKind::DeclareAttackers { .. }));
    (g, ask, bears)
}

fn limits(request: &Request) -> Vec<(Target, u32)> {
    let RequestKind::DeclareAttackers { requirements, .. } = &request.kind else { panic!("not a declaration of attackers") };
    requirements.limits.clone()
}

/// The one planeswalker P1 may attack.
fn walker(request: &Request) -> ObjectId {
    match &request.kind {
        RequestKind::DeclareAttackers { planeswalkers, .. } if planeswalkers.len() == 1 => planeswalkers[0],
        other => panic!("one planeswalker to attack, not {other:?}"),
    }
}

fn legal(g: &Game, request: &Request, attacks: &[(ObjectId, Target)]) -> bool {
    crate::check_answer(g, request, &Answer::Attack(attacks.to_vec())).is_ok()
}

/// The cap is in the declaration the player is asked for, and one creature may attack the planeswalker, a second may not.
#[test]
fn a_second_creature_cannot_attack_a_capped_planeswalker() {
    let (g, ask, bears) = game(&[(&CAPPED, false)], 2);
    let w = Target::Object(walker(&ask));
    assert_eq!(limits(&ask), vec![(w, 1)]);
    assert!(legal(&g, &ask, &[(bears[0], w)]));
    assert!(legal(&g, &ask, &[(bears[0], w), (bears[1], Target::Player(P0))]));
    assert!(!legal(&g, &ask, &[(bears[0], w), (bears[1], w)]));
}

/// Two caps on one target: the smallest applies, and two may attack where the only cap allows two.
#[test]
fn the_smallest_of_two_limits_applies() {
    let (g, ask, bears) = game(&[(&CAPPED, false), (&CAPPED_TWO, false)], 2);
    let w = Target::Object(walker(&ask));
    assert_eq!(limits(&ask), vec![(w, 1)]);
    assert!(!legal(&g, &ask, &[(bears[0], w), (bears[1], w)]));
    let (g, ask, bears) = game(&[(&CAPPED_TWO, false)], 2);
    let w = Target::Object(walker(&ask));
    assert_eq!(limits(&ask), vec![(w, 2)]);
    assert!(legal(&g, &ask, &[(bears[0], w), (bears[1], w)]));
}

/// The rule is asked with the permanent that has it as its source and that permanent's controller.
#[test]
fn the_ability_is_asked_with_its_permanents_asker() {
    let (g, ask, _) = game(&[(&RECORDER, false)], 1);
    let source = ASKED_SOURCE.load(Ordering::Relaxed);
    assert_ne!(source, u32::MAX, "the rule was asked");
    assert_eq!(g.state.obj(ObjectId(source)).card, g.cards.id_of(&RECORDER));
    assert_eq!(ASKED_CONTROLLER.load(Ordering::Relaxed), u32::from(P0.0));
    assert!(limits(&ask).is_empty());
}

/// A permanent that has lost all its abilities has none of these (CR 613.1f): its cap is gone.
#[test]
fn a_source_that_lost_all_abilities_caps_nothing() {
    let (g, ask, bears) = game(&[(&CAPPED, true)], 2);
    let w = Target::Object(walker(&ask));
    assert!(limits(&ask).is_empty());
    assert!(legal(&g, &ask, &[(bears[0], w), (bears[1], w)]));
}
