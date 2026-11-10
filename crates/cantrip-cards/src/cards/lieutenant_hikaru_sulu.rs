use cantrip_api::*;
use cantrip_mechanics::{effects, explore, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever Lieutenant Sulu becomes tapped, he explores." (CR 701.44a, 603.2): the trigger is the event that taps him, by
/// whatever path taps him - an effect, a cost, or being declared as an attacker (CR 508.1f) - so it fires once each time he
/// becomes tapped, and the explore happens as it resolves (CR 603.2e): its controller reveals the top card of their library,
/// which a land card goes to their hand for and a nonland card grows him by a +1/+1 counter before they may put it into
/// their graveyard (CR 701.44a). His controller is read from the permanent's last known information if he has left the
/// battlefield by then (CR 701.44c).
fn explores(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    explore(g, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_BECOMES_TAPPED, explores, "Whenever Lieutenant Sulu becomes tapped, he explores."),
        ability(cost("{2}{U}"), effects::target_creature_cant_be_blocked_this_turn, "{2}{U}: Target creature can't be blocked this turn.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::LIEUTENANT_HIKARU_SULU
};
