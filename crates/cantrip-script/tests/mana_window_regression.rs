use cantrip_engine::{PlayerId, RequestKind};
use cantrip_script::{Advance, GameScript, Scripted};

fn script(actions: &str) -> String {
    format!(r#"[Bot1 Deck]
Mountain
Mox Ruby
Mox Emerald
Lightning Bolt
Lightning Bolt
Island x15

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.Mountain.Play
Bot1.Hand.Mox Ruby.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Emerald.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Mox Ruby.Activate
{actions}
"#)
}

fn run_both(name: &str, text: &str) {
    let s = GameScript::parse(text, name).unwrap();
    let alone = cantrip_script::run(&s).unwrap_or_else(|e| panic!("{}", e.report));
    let expected = alone.dump.render();
    let mut driven = Scripted::deal(s).unwrap();
    loop {
        match driven.advance(&[PlayerId(0), PlayerId(1)]).unwrap_or_else(|e| panic!("{}", e.report)) {
            Advance::Ask(ask) => driven.answered(ask.intent).unwrap_or_else(|e| panic!("{}", e.report)),
            Advance::Check(_) => {},
            Advance::Done => break,
        }
    }
    assert_eq!(driven.verdict().unwrap().render(), expected);
}

#[test]
fn funded_cast_finishes_before_checks_and_next_mana_ability() {
    run_both("covered-then-next-mana", &script(r#"Bot1.Hand.Lightning Bolt.Cast(Bot2)
? Bot1.Mana =
? Stack.Count = 1
Bot1.Battlefield.Mox Emerald.Activate
? Bot1.Mana = {G}
? Bot1.ManaFrom = {G} Mox Emerald
Bot1.Battlefield.Mountain.Activate
Bot1.Hand.Lightning Bolt.Cast(Bot2)
? Bot1.Mana = {G}
? Bot1.ManaFrom = {G} Mox Emerald
? Stack.Count = 2
"#));
}






#[test]
fn attack_payment_refuses_the_tapped_attacker_and_can_be_cancelled() {
    let original = r#"# The chosen attackers become tapped (CR 508.1f) before any cost to attack is determined and paid (CR 508.1h-j), so a
# creature cannot tap for mana to pay its own way past Ghostly Prison. On turn 3 Bot1's only mana is the Llanowar Elves
# it declares as an attacker: while the {2} is owed its mana ability is not offered, nothing can pay, and Bot1 gives the
# declaration up. The Elves untap with it (CR 733), and the declaration made again is no attack at all: with no mana a
# taxed attack cannot be made. Bot2 stays at 20 and the Elves end untapped.

[Bot1 Deck]
Lotus Petal
Llanowar Elves
Island
Island
Island
Island
Island
Island x10

[Bot2 Deck]
Plains
Black Lotus
Ghostly Prison
Plains
Plains
Plains
Plains
Plains x10

[Actions]
Bot1.Hand.Lotus Petal.Cast
Bot1.Battlefield.Lotus Petal.Activate(Color: Green)
Bot1.Hand.Llanowar Elves.Cast
[T2 Bot2.Main1]
Bot2.Hand.Black Lotus.Cast
Bot2.Battlefield.Black Lotus.Activate(Color: White)
Bot2.Hand.Ghostly Prison.Cast
[T3 Bot1.Main1]
Bot1.Attack(Llanowar Elves)
? Combat.AttackCost = {2}
! Bot1.Battlefield.Llanowar Elves.Activate
Bot1.Cancel
[T3 Bot1.Main2]
? Bot1.Battlefield.Llanowar Elves.Tapped = false
Bot1.Battlefield.Llanowar Elves.Activate
? Bot1.Mana = {G}

[Expect]
Turn = 3
Bot2.Life = 20
Bot1.Battlefield.Llanowar Elves.Tapped = true
Combat.Attackers.Count = 0
"#;
    run_both("attack-tax-refusal", original);
    let illegal = original.replace("! Bot1.Battlefield.Llanowar Elves.Activate", "Bot1.Battlefield.Llanowar Elves.Activate");
    let script = GameScript::parse(&illegal, "attack-tax-illegal-mana").unwrap();
    assert!(cantrip_script::run(&script).is_err(), "a tapped attacker cannot pay its own attack cost");
}


#[test]
fn mana_ability_is_not_offered_after_the_player_starts_paying() {
    let text = r#"[Bot1 Deck]
Mountain
Mox Ruby
Mox Jet
Mox Emerald
Ornithopter
Fling
Island
Island x15

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.Mountain.Play
Bot1.Hand.Mox Ruby.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Jet.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Mox Emerald.Cast
Bot1.Pass
Bot2.Pass
Bot1.Hand.Ornithopter.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Mox Ruby.Activate
Bot1.Battlefield.Mox Jet.Activate
Bot1.Battlefield.Mox Emerald.Activate
Bot1.Hand.Fling.Cast(Bot2)
! Bot1.Battlefield.Mountain.Activate
Bot1.Cancel
? Bot1.Battlefield.Mountain.Tapped = false
? Bot1.Mana = {B}{R}{G}
? Stack.Count = 0
"#;
    run_both("mana-after-payment-started", text);
    let mut driven = Scripted::deal(GameScript::parse(text, "mana-after-payment-boundary").unwrap()).unwrap();
    loop {
        match driven.advance(&[PlayerId(0), PlayerId(1)]).unwrap() {
            Advance::Ask(ask) if ask.line.as_deref().is_some_and(|line| line.contains("Bot1.Cancel")) => {
                assert!(matches!(ask.request.kind, RequestKind::PayCosts { mana_abilities: false, .. }));
                break;
            }
            Advance::Ask(ask) => driven.answered(ask.intent).unwrap(),
            Advance::Check(_) => {},
            Advance::Done => panic!("the cancellation question was not reached"),
        }
    }
    let illegal = text.replace("! Bot1.Battlefield.Mountain.Activate", "Bot1.Battlefield.Mountain.Activate");
    let script = GameScript::parse(&illegal, "mana-after-payment-illegal").unwrap();
    assert!(cantrip_script::run(&script).is_err(), "mana abilities cannot be activated after cost payment starts");
}

#[test]
fn covered_payment_completes_across_special_actions_and_step_boundaries() {
    let future_refusal = script(r#"Bot1.Hand.Lightning Bolt.Cast(Bot2)
? Bot1.Mana =
? Stack.Count = 1
! Bot1.Battlefield.Mox Ruby.Activate
"#);
    for (name, source) in [
        ("face-up-handover", include_str!("../../../scenarios/abhorrent-oculus-manifested-creature-card-turns-face-up-for-its-mana-cost.cantrip")),
        ("face-up-explicit-assign", include_str!("../../../scenarios/dress-down-does-not-stop-a-manifested-creature-turning-face-up.cantrip")),
        ("attack-tax-step", include_str!("../../../scenarios/embercleave-counts-the-attackers-paid-for-under-ghostly-prison.cantrip")),
        ("effect-cast", include_str!("../../../scenarios/doc-aurlock-does-not-reduce-a-spell-cast-from-an-opponents-graveyard.cantrip")),
        ("miracle-cast", include_str!("../../../scenarios/molecule-man-and-a-cards-own-miracle-the-player-may-take-its-own.cantrip")),
        ("future-refusal", future_refusal.as_str()),
        ("future-refusal-on-mana-ability", include_str!("../../../scenarios/foraging-wickermaw-becomes-the-color-of-its-mana-for-the-turn-once-a-turn.cantrip")),
    ] {
        run_both(name, source);
    }
}


#[test]
fn named_object_choice_uses_unambiguous_mana_before_its_question() {
    let text = include_str!("../../../scenarios/embrace-oblivion-can-destroy-its-controllers-own-creature-when-another-permanent-is-sacrificed.cantrip");
    assert!(!text.contains("Bot1.Choose(Pay mana)"));
    run_both("named-object-after-mana", text);
}


#[test]
fn refused_effect_choice_is_not_a_cost_order_question() {
    let text = include_str!("../../../scenarios/eerie-gravestone-with-no-creature-among-the-four-milled-asks-nothing.cantrip");
    assert!(text.contains("! Bot1.Choose"));
    run_both("refused-effect-choice-after-payment", text);
}

#[test]
fn source_names_do_not_hide_distinct_additional_costs() {
    let text = include_str!("../../../scenarios/fling-casualty-caster-orders-the-costs.cantrip");
    run_both("explicit-additional-cost-sources", text);
}


