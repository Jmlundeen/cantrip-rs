//! A cap on how many creatures may attack a player (CR 508.1c: "No more than one creature can attack you each combat")
//! in the AI player's terms: the ask carries it (`AskKind::DeclareAttackers::max_attackers`, as blocks carry
//! `max_blockers`), so no walk through the factoring declares one creature more than the cap allows, and none is
//! refused for anything but the most requirements the validator judges (`Rejection::global`). The position is a
//! scenario's declaration of attackers with three Ornithopters; the cap and a requirement are put into its question as
//! a static ability and an effect would put them.

use cantrip_ai::{Answering, Knowledge, observe_indexed};
use cantrip_engine::*;
use cantrip_obs::factor::Machine;
use cantrip_obs::{AskKind, InfoMode};
use cantrip_script::GameScript;

const SCRIPT: &str = "\
[Bot1 Deck]
Ornithopter x3
Island x10

[Bot2 Deck]
Island x10

[Actions]
Bot1.Hand.Ornithopter.Cast
Bot1.Hand.Ornithopter.Cast
Bot1.Hand.Ornithopter.Cast
[T3 Bot1.Main1]
[Handover Bot1.Attack]
";

#[test]
fn walks_never_declare_more_attackers_than_a_cap_allows() {
    let script = GameScript::parse(SCRIPT, "attack-caps").unwrap();
    let game = cantrip_script::run(&script).unwrap_or_else(|f| panic!("{f:?}")).game;
    let mut request = game.pending().cloned().expect("a declaration of attackers");
    let RequestKind::DeclareAttackers { eligible, players, requirements, costs, .. } = &mut request.kind else {
        panic!("not a declaration of attackers: {request:?}")
    };
    assert_eq!(eligible.len(), 3);
    let defender = Target::Player(players[0]);
    // One creature at most may attack the defending player, and the last Ornithopter attacks each combat if able.
    requirements.limits = vec![(defender, 1)];
    requirements.required = vec![RequiredAttack { attacker: eligible[2], aim: AttackAim::Anything }];
    (requirements.most, requirements.suggested) = requirements.solve(eligible, &[defender], costs);
    assert_eq!(requirements.most, 1);
    let player = request.player;
    let knowledge = Knowledge::of(&game, player);
    let (obs, index) = observe_indexed(&game, &knowledge, player, InfoMode::Full).expect("an observation");
    let answering = Answering::new(&game, &request, &obs, &index);
    let AskKind::DeclareAttackers { max_attackers, .. } = &answering.ask.kind else {
        panic!("not an attack ask: {:?}", answering.ask.kind)
    };
    assert_eq!(max_attackers, &vec![1]);
    // Every walk through the machine: each step's choice by the walk's number, digit by digit.
    let (mut accepted, mut global) = (0, 0);
    for walk in 0..27usize {
        let mut m = Machine::new(&answering.ask, &[]);
        let mut digits = walk;
        while let Some(step) = m.step() {
            let legal: Vec<usize> = step.legal().collect();
            assert!(!legal.is_empty(), "a step with nothing legal: {step:?}");
            m.choose(legal[digits % legal.len()]);
            digits /= 3;
        }
        let walked = m.answer().unwrap().clone();
        match answering.engine(&game, &request, &walked) {
            Ok(Answer::Attack(attacks)) => {
                assert!(attacks.len() <= 1, "{attacks:?}");
                accepted += 1;
            }
            Ok(other) => panic!("not an attack: {other:?}"),
            Err(r) => {
                assert!(r.global(), "refused for the factoring's fault: {} ({:?})", r.why, r.answer);
                global += 1;
            }
        }
    }
    assert!(accepted > 0 && global > 0, "accepted {accepted}, refused for the most requirements {global}");
}
