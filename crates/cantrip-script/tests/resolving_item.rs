//! What a resolving spell or ability does after it casts a spell (CR 608.2g) is still its own doing: the spell cast goes
//! on the stack above it, and the item resolving is the one its resolution began with (`FrameKind::Resolve`), not the top
//! of the stack. Ripple through Thrumming Stone reveals four cards, casts the two Lightning Bolts among them, and puts the
//! rest on the bottom: all of it, the second cast made with the first Bolt on the stack above the ability included, is the
//! ripple ability's doing, its cause in the history (CR 702.60a). Putting the rest on the bottom of the library they are
//! in moves no card to another zone.

use cantrip_engine::*;
use cantrip_script::{GameScript, run};

const RIPPLE: &str = "
[Bot1 Deck]
Mountain
Sol Ring
Black Lotus
Thrumming Stone
Lightning Bolt
Mountain
Island
Island
Lightning Bolt
Lightning Bolt
Island
Island
Island x20
[Bot2 Deck]
Island x20
[Actions]
Bot1.Hand.Mountain.Play
Bot1.Battlefield.Mountain.Activate
Bot1.Hand.Sol Ring.Cast
Bot1.Hand.Black Lotus.Cast
Bot1.Battlefield.Sol Ring.Activate
Bot1.Battlefield.Black Lotus.Activate(Color: Red)
Bot1.Hand.Thrumming Stone.Cast
[T3 Bot1.Main1]
Bot1.Battlefield.Mountain.Activate
Bot1.Hand.Lightning Bolt.Cast(Bot2)
Bot1.Accept
Bot1.Choose(Lightning Bolt)
Bot1.Target(Bot2)
Bot1.Choose(Lightning Bolt)
Bot1.Target(Bot2)
[Bot1.Main2]
[Expect]
Bot2.Life = 11
Bot1.Graveyard.Count = 4
Bot1.Library.Count = 22
";

#[test]
fn what_ripple_does_after_casting_is_still_the_ripple_abilitys() {
    let script = GameScript::parse(RIPPLE, "ripple-casts-then-bottoms").expect("parses");
    let result = match run(&script) {
        Ok(result) => result,
        Err(failure) => panic!("{}", failure.report),
    };
    let log = &result.game.log;
    let first = log.iter().position(|r| matches!(r.event, Event::Revealed { .. })).expect("ripple revealed");
    let ripple = log[first].cause;
    assert!(matches!(ripple, Cause::Ability { .. }), "the reveal is the ripple ability's: {ripple:?}");
    assert_eq!(log[first..].iter().filter(|r| matches!(r.event, Event::Revealed { .. }) && r.cause == ripple).count(), 4);
    // Its run of the history: from the reveal to the last thing it did, the two casts among them and nothing else's
    // doing - the second cast, made with the first Bolt on top of the stack, is the ripple ability's too.
    let last = log.iter().rposition(|r| r.cause == ripple).expect("the ripple ability's records");
    let run = &log[first..=last];
    assert!(run.iter().all(|r| r.cause == ripple), "everything from the reveal on is the ripple ability's doing");
    assert_eq!(run.iter().filter(|r| matches!(r.event, Event::SpellCast { .. })).count(), 2, "both Bolts cast by it");
}
