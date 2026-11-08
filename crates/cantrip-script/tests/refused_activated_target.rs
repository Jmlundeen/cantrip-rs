//! A refused target on an activated ability (CR 601.2c, 602.2b): targets are chosen as the ability is announced, so the
//! ability is offered whatever the target - a refusal naming one (`! Bot1.Battlefield.Spot, Felis catus.Activate(Target: ...)`)
//! holds when no option of that ability could take it, and fails when one could. A named target on an action with no target
//! slot proves nothing. Spot's "{4}, Sacrifice a Food: Destroy target artifact" is the printed case: a creature is no artifact,
//! so it is refused, and the artifact the same script then destroys is taken.

use std::sync::OnceLock;

use cantrip_engine::*;
use cantrip_script::{GameScript, run_on};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all()))
}

/// The Spot scenario with `REFUSAL` replaced by the refusal under test, or by nothing.
fn scenario(refusal: &str) -> String {
    const TEXT: &str = "
[Bot1 Deck]
Spot, Felis catus
Mox Pearl
Mox Sapphire
Mox Jet
Mox Ruby
Mox Emerald
Plains x14

[Bot2 Deck]
Island x20

[Actions]
Bot1.Hand.Mox Pearl.Cast
Bot1.Hand.Mox Sapphire.Cast
Bot1.Hand.Mox Jet.Cast
Bot1.Hand.Mox Ruby.Cast
Bot1.Hand.Mox Emerald.Cast
Bot1.Battlefield.Mox Pearl.Activate
Bot1.Battlefield.Mox Sapphire.Activate
Bot1.Battlefield.Mox Jet.Activate
Bot1.Hand.Spot, Felis catus.Cast
? Bot1.Mana =
Bot1.Pass
Bot2.Pass
Bot1.Pass
Bot2.Pass
? Bot1.Battlefield.Food Token.Types = Artifact
[T3 Bot1.Main1]
Bot1.Hand.Plains.Play
Bot1.Battlefield.Plains.Activate
Bot1.Battlefield.Mox Pearl.Activate
Bot1.Battlefield.Mox Sapphire.Activate
Bot1.Battlefield.Mox Jet.Activate
REFUSAL
Bot1.Battlefield.Spot, Felis catus.Activate(Target: Bot1.Battlefield.Mox Ruby)
! Bot1.Choose(Mox Ruby)
Bot1.Choose(Food Token)
? Bot1.Mana =
Bot1.Pass
Bot2.Pass
? Bot1.Battlefield.Mox Ruby = (absent)

[Expect]
Turn = 3
Bot1.Battlefield.Mox Ruby = (absent)
Bot1.Battlefield.Food Token = (absent)
Bot1.Battlefield.Spot, Felis catus.Types = Creature
Bot1.Graveyard = Mox Ruby
";
    let line = if refusal.is_empty() { String::new() } else { format!("{refusal}
") };
    TEXT.replace("REFUSAL
", &line)
}

fn run(text: &str) -> Result<(), String> {
    let script = GameScript::parse(text, "refused-activated-target").unwrap_or_else(|e| panic!("{e}"));
    match run_on(&script, table()) {
        Ok(_) => Ok(()),
        Err(failure) => Err(failure.message),
    }
}

/// The script itself, with no refusal in it, passes: the fixture is the card's own.
#[test]
fn the_script_passes_with_no_refusal() {
    if let Err(message) = run(&scenario("")) {
        panic!("{message}");
    }
}

/// A creature is no artifact, so Spot's target slot cannot take it: the refusal holds, the ability stays offered, and the
/// same script then activates it on the artifact, which is a legal target.
#[test]
fn a_target_the_filter_refuses_holds_and_the_ability_is_still_offered() {
    if let Err(message) = run(&scenario("! Bot1.Battlefield.Spot, Felis catus.Activate(Target: Spot, Felis catus)")) {
        panic!("{message}");
    }
}

/// The artifact the filter takes is offered, so the same line written about it fails - the refusal is judged, not swallowed.
#[test]
fn a_target_the_filter_takes_fails_the_refusal() {
    let message = run(&scenario("! Bot1.Battlefield.Spot, Felis catus.Activate(Target: Bot1.Battlefield.Mox Ruby)"))
        .expect_err("a refusal of a legal target must fail");
    assert!(message.contains("must not be offered, but it is"), "{message}");
    assert!(message.contains("Mox Ruby may be chosen"), "{message}");
}
