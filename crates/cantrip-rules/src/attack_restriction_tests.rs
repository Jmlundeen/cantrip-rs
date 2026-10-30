//! A static ability that restricts attacking (CR 508.1c, `CardDef::attack_rules`), asked of each attack a creature could
//! make: "creatures with flying can't attack you" (Storm, Windrider; Blazing Archon's "creatures can't attack you") forbids
//! an attack on its controller and leaves their planeswalkers open (CR 506.2, 508.1b); "... can't attack you or
//! planeswalkers you control" (Sandwurm Convergence, the Vows) forbids both, and a creature forbidden every attack is not
//! eligible to attack at all. A requirement counts only the attacks left (CR 508.1d). Over test cards.

use crate::{Game, Setup};
use cantrip_api::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

static FILLER: CardDef = CardDef { name: "Attack Rule Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
static BEAR: CardDef =
    CardDef { name: "Attack Rule Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };
static FLIER: CardDef = CardDef {
    name: "Attack Rule Flier",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..CardDef::BLANK
};
/// Enters with its printed loyalty (CR 306.5b), so it stays on the battlefield to be attacked.
static WALKER: CardDef = CardDef { name: "Attack Rule Walker", types: TypeSet::of(&[CardType::Planeswalker]), loyalty: Some(3), ..CardDef::BLANK };

/// "Creatures with flying can't attack you."
fn fliers_cant_attack_you(q: &dyn Query, asker: &Asker, creature: ObjectId, at: Target) -> bool {
    at != Target::Player(asker.controller) || !q.chars(creature).has(Keyword::Flying)
}

/// "Creatures with flying can't attack you or planeswalkers you control."
fn fliers_cant_attack_you_or_your_planeswalkers(q: &dyn Query, asker: &Asker, creature: ObjectId, at: Target) -> bool {
    let yours = match at {
        Target::Player(p) => p == asker.controller,
        Target::Object(id) => q.chars(id).is(CardType::Planeswalker) && q.controller(id) == asker.controller,
        _ => false,
    };
    !yours || !q.chars(creature).has(Keyword::Flying)
}

static STORM: CardDef =
    CardDef { name: "Attack Rule You", types: TypeSet::of(&[CardType::Enchantment]), attack_rules: &[fliers_cant_attack_you], ..CardDef::BLANK };
static CONVERGENCE: CardDef = CardDef {
    name: "Attack Rule You Or Walkers",
    types: TypeSet::of(&[CardType::Enchantment]),
    attack_rules: &[fliers_cant_attack_you_or_your_planeswalkers],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    Box::leak(Box::new(CardTable::new([&FILLER, &BEAR, &FLIER, &WALKER, &STORM, &CONVERGENCE])))
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

/// A game of two: `restriction` on P0's battlefield (if it has lost all its abilities, `lost`), P0's Walker cast in its
/// first main phase, and for P1 `creatures` (each goaded by P0, if `goaded`) - then the declaration of attackers P1 is
/// asked for on its own turn, or none where P1 has nothing that may attack.
fn game(restriction: &'static CardDef, lost: bool, creatures: &[&'static CardDef], goaded: bool) -> (Game, Option<Request>, Vec<ObjectId>) {
    let table = table();
    let filler = table.id_of(&FILLER);
    let mut deck = vec![table.id_of(&WALKER)];
    deck.extend(std::iter::repeat_n(filler, 30));
    let mut g = Game::new(table, &Setup { decks: vec![deck, vec![filler; 30]], ..Default::default() });
    g.verify = true;
    let id = g.state.push_obj(Obj::new(table.id_of(restriction), P0, Zone::BATTLEFIELD));
    g.state.zone_push(Zone::BATTLEFIELD, id);
    if lost {
        let timestamp = g.state.timestamp();
        g.state.effect_push(ContinuousEffect { source: id, controller: P0, timestamp, affected: id, kind: EffectKind::LoseAllAbilities, duration: Duration::Indefinite });
    }
    let creatures: Vec<ObjectId> = creatures
        .iter()
        .map(|&def| {
            let id = g.state.push_obj(Obj::new(table.id_of(def), P1, Zone::BATTLEFIELD));
            g.state.zone_push(Zone::BATTLEFIELD, id);
            if goaded {
                let timestamp = g.state.timestamp();
                g.state.effect_push(ContinuousEffect { source: id, controller: P0, timestamp, affected: id, kind: EffectKind::Goaded, duration: Duration::Indefinite });
            }
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
    // P1's turn: its declaration of attackers, or its end step's priority where it is never asked to declare.
    let ask = until(&mut g, |g, r| {
        matches!(r.kind, RequestKind::DeclareAttackers { .. }) || (g.state.turn.active == P1 && g.state.turn.step() == Step::End)
    });
    let ask = matches!(ask.kind, RequestKind::DeclareAttackers { .. }).then_some(ask);
    (g, ask, creatures)
}

/// The one planeswalker P1 may attack.
fn walker(g: &Game) -> Target {
    let id = g.state.battlefield().iter().copied().find(|&id| g.state.obj(id).card == g.cards.id_of(&WALKER)).expect("the Walker");
    Target::Object(id)
}

fn legal(g: &Game, request: &Request, attacks: &[(ObjectId, Target)]) -> bool {
    crate::check_answer(g, request, &Answer::Attack(attacks.to_vec())).is_ok()
}

fn requirements(request: &Request) -> &AttackRules {
    let RequestKind::DeclareAttackers { requirements, .. } = &request.kind else { panic!("not a declaration of attackers") };
    requirements
}

/// "Creatures with flying can't attack you": the flier may attack the planeswalker, not the player, and the attack the
/// rule forbids is in the question.
#[test]
fn a_flier_may_attack_the_planeswalker_but_not_the_player() {
    let (g, ask, c) = game(&STORM, false, &[&FLIER], false);
    let ask = ask.expect("the flier may attack");
    let (flier, w) = (c[0], walker(&g));
    assert_eq!(requirements(&ask).barred, vec![(flier, Target::Player(P0))]);
    assert!(legal(&g, &ask, &[(flier, w)]));
    assert!(!legal(&g, &ask, &[(flier, Target::Player(P0))]));
    assert!(legal(&g, &ask, &[]));
    assert!(g.attack_rules_allow(flier));
    // Only the attack left is offered to be added.
    let verdict = requirements(&ask).judge(&[flier], &[Target::Player(P0), w], &[], &[], true);
    assert_eq!(verdict.addable, vec![(flier, w)]);
}

/// A creature without flying is under neither restriction.
#[test]
fn a_creature_without_flying_is_unaffected() {
    for restriction in [&STORM, &CONVERGENCE] {
        let (g, ask, c) = game(restriction, false, &[&BEAR], false);
        let ask = ask.expect("the bear may attack");
        assert!(requirements(&ask).barred.is_empty());
        assert!(legal(&g, &ask, &[(c[0], Target::Player(P0))]));
        assert!(legal(&g, &ask, &[(c[0], walker(&g))]));
    }
}

/// "... can't attack you or planeswalkers you control": the flier has nothing left to attack, so it is not eligible and
/// both attacks are refused; the bear beside it attacks either.
#[test]
fn or_planeswalkers_you_control_forbids_both() {
    let (g, ask, c) = game(&CONVERGENCE, false, &[&FLIER, &BEAR], false);
    let ask = ask.expect("the bear may attack");
    let (flier, bear, w) = (c[0], c[1], walker(&g));
    let RequestKind::DeclareAttackers { eligible, .. } = &ask.kind else { unreachable!() };
    assert_eq!(eligible, &vec![bear]);
    assert!(!legal(&g, &ask, &[(flier, Target::Player(P0))]));
    assert!(!legal(&g, &ask, &[(flier, w)]));
    assert!(legal(&g, &ask, &[(bear, Target::Player(P0))]));
    assert!(legal(&g, &ask, &[(bear, w)]));
    assert!(!g.attack_rules_allow(flier));
    // Alone, the flier is not asked to declare at all.
    let (_, ask, _) = game(&CONVERGENCE, false, &[&FLIER], false);
    assert!(ask.is_none());
}

/// A requirement counts only the attacks a restriction leaves (CR 508.1d): a flier goaded by P0 must attack if able, and
/// a player other than P0 is none in a game of two, so it obeys the most attacking the planeswalker; staying home is
/// refused, and attacking P0 is forbidden. Under "or planeswalkers you control" it can attack nothing, and nothing is
/// required of it.
#[test]
fn a_requirement_is_obeyed_only_by_an_attack_left_open() {
    let (g, ask, c) = game(&STORM, false, &[&FLIER], true);
    let ask = ask.expect("the flier may attack");
    let (flier, w) = (c[0], walker(&g));
    assert_eq!(requirements(&ask).most, 1);
    assert_eq!(requirements(&ask).suggested, vec![(flier, w)]);
    assert!(legal(&g, &ask, &[(flier, w)]));
    assert!(!legal(&g, &ask, &[]));
    assert!(!legal(&g, &ask, &[(flier, Target::Player(P0))]));
    let (_, ask, _) = game(&CONVERGENCE, false, &[&FLIER], true);
    assert!(ask.is_none());
}

/// A permanent that has lost all its abilities has none of these (CR 613.1f): the flier attacks the player.
#[test]
fn a_source_that_lost_all_abilities_forbids_nothing() {
    let (g, ask, c) = game(&CONVERGENCE, true, &[&FLIER], false);
    let ask = ask.expect("the flier may attack");
    assert!(requirements(&ask).barred.is_empty());
    assert!(legal(&g, &ask, &[(c[0], Target::Player(P0))]));
}
