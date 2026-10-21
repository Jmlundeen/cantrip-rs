//! Combat with more creatures than eight or sixteen bits count, which the event cap (RX-1) lets a game reach: 300 Grizzly
//! Bears attack, the attack is recorded as 300 attackers (CR 508.1), and 300 Llanowar Elves block one of them, which is legal
//! (CR 509.1b: a creature with no restriction can be blocked by any number of creatures).

use std::sync::OnceLock;

use cantrip_engine::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const N: usize = 300;

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all()))
}

#[test]
fn three_hundred_attackers_are_counted_and_three_hundred_blockers_on_one_are_legal() {
    let card = |name: &str| table().find(name).unwrap_or_else(|| panic!("no card {name}"));
    let deck = || vec![card("Plains"); 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck(), deck()], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    for owner in [P0, P1] {
        for _ in 0..N {
            let id = game.state.push_obj(Obj::new(card("Llanowar Elves"), owner, Zone::BATTLEFIELD));
            game.state.zone_push(Zone::BATTLEFIELD, id);
        }
    }
    let mut blocked = None;
    for _ in 0..2000 {
        game.run();
        let Some(request) = game.pending().cloned() else { break };
        // The blocks declared: the step's first priority.
        if blocked.is_some() {
            break;
        }
        let answer = match &request.kind {
            RequestKind::DeclareAttackers { eligible, .. } if request.player == P0 && eligible.len() == N => {
                Answer::Attack(eligible.iter().map(|&a| (a, Target::Player(P1))).collect())
            }
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { eligible, attackers, .. } if request.player == P1 && attackers.len() == N => {
                assert_eq!(eligible.len(), N);
                blocked = Some(attackers[0]);
                Answer::Block(eligible.iter().map(|&b| (b, attackers[0])).collect())
            }
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            _ => Answer::Pass,
        };
        game.answer(request.id, request.player, answer).unwrap_or_else(|e| panic!("{:?} refused: {e:?}", request.kind));
    }
    let attacker = blocked.expect("300 attackers were declared and blocked");
    assert!(game.log.iter().any(|r| matches!(r.event, Event::AttackersDeclared { player: P0, count } if count as usize == N)));
    assert_eq!(game.state.blocks.iter().filter(|b| b.attacker == attacker).count(), N, "every blocker blocks the one attacker");
}
