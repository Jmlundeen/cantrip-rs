//! A cost to attack is not a requirement (CR 508.1d): a creature that attacks each combat if able need not attack a
//! player whom attacking costs something (Ghostly Prison), and a seat that cannot pay may leave it home. The AI player's
//! factoring of a declaration of attackers must offer that, or a seat with no mana is made to attack, cannot pay, has the
//! declaration taken back (CR 508.1h-j, `turn.rs`) and is asked again - the same answer forever.

use cantrip_ai::eval::{Evaluator, FeatureBatch, Outputs};
use cantrip_ai::net::{self, Choice};
use cantrip_ai::record::table;
use cantrip_ai::{EvalError, Knowledge};
use cantrip_engine::*;
use cantrip_obs::InfoMode;
use cantrip_script::GameScript;

/// Every option alike: the first legal one is taken at every step.
struct Uniform;

impl Evaluator for Uniform {
    fn evaluate(&self, b: &FeatureBatch) -> Result<Outputs, EvalError> {
        Ok(Outputs {
            value: vec![0.0; b.rows],
            policy: b.options.iter().map(|&(_, n)| vec![0.0; n]).collect(),
            aux: vec![[0.0; 2]; b.rows],
        })
    }
}

/// Bot1 has Anje's Ravager ("attacks each combat if able") and no mana left; Bot2 has Ghostly Prison, so attacking Bot2
/// costs {2} and nothing else may be attacked.
const SCRIPT: &str = "\
[Bot1 Deck]
Black Lotus
Anje's Ravager
Island x16

[Bot2 Deck]
Plains
Ghostly Prison
Mox Pearl
Mox Jet
Plains x14

[Actions]
Bot1.Hand.Black Lotus.Cast
Bot1.Battlefield.Black Lotus.Activate(Color: Red)
Bot1.Hand.Anje's Ravager.Cast
[T2 Bot2.Main1]
Bot2.Hand.Plains.Play
Bot2.Hand.Mox Pearl.Cast
Bot2.Hand.Mox Jet.Cast
Bot2.Battlefield.Plains.Activate
Bot2.Battlefield.Mox Pearl.Activate
Bot2.Battlefield.Mox Jet.Activate
Bot2.Hand.Ghostly Prison.Cast
[T3 Bot1.Main1]
[Handover Bot1.Attack]

[Expect]
Turn = 3
Step = Attackers
";

#[test]
fn a_seat_that_cannot_pay_to_attack_leaves_a_must_attack_creature_home() {
    let script = GameScript::parse(SCRIPT, "attack-cost-no-mana").expect("the script parses");
    let result = cantrip_script::run(&script).unwrap_or_else(|f| panic!("{}\n{}", f.message, f.report));
    assert!(result.handed_over, "handed over at the declaration of attackers");
    let mut game = result.game;
    let request = game.pending().cloned().expect("a declaration of attackers");
    let RequestKind::DeclareAttackers { costs, requirements, .. } = &request.kind else {
        panic!("not a declaration of attackers: {request:?}")
    };
    assert!(!costs.is_empty() && !requirements.required.is_empty() && requirements.most == 0);
    // The seats answer as the AI player does until Bot1's turn is over; a declaration taken back again and again never
    // gets there.
    let mut declarations = 0;
    for _ in 0..200 {
        if game.state.turn.number > 3 {
            break;
        }
        let request = game.pending().cloned().expect("a request");
        if matches!(request.kind, RequestKind::DeclareAttackers { .. }) {
            declarations += 1;
        }
        let knowledge = Knowledge::of(&game, request.player);
        let decision = net::answer(&game, &request, &knowledge, InfoMode::Full, table(), &Uniform, Choice::Best).expect("an answer");
        game.answer(request.id, request.player, decision.answer).expect("the answer is taken");
        game.run();
    }
    assert!(game.state.turn.number > 3, "Bot1's turn never ended: {declarations} declarations of attackers");
    assert_eq!(declarations, 1);
}
